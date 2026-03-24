use std::{
    net::SocketAddr,
    sync::{Arc, LazyLock, Mutex},
};

use defguard_version::{
    DefguardComponent, Version,
    server::{DefguardVersionLayer, grpc::DefguardVersionInterceptor},
};
use tokio::sync::{Notify, mpsc, oneshot};
use tokio_stream::wrappers::UnboundedReceiverStream;
use tonic::{
    Request, Response, Status,
    transport::{Identity, Server, ServerTlsConfig},
};

use crate::{
    CommsChannel, LogsReceiver, MIN_CORE_VERSION, VERSION,
    acme::{Port80Permit, run_acme_http01},
    error::ApiError,
    grpc::Configuration,
    proto::{
        AcmeCertificate, AcmeChallenge, AcmeIssueEvent, AcmeProgress, AcmeStep, CertificateInfo,
        DerPayload, LogEntry, acme_issue_event, proxy_setup_server,
    },
};

static SETUP_CHANNEL: LazyLock<CommsChannel<Option<Configuration>>> = LazyLock::new(|| {
    let (tx, rx) = mpsc::channel(10);
    (
        Arc::new(tokio::sync::Mutex::new(tx)),
        Arc::new(tokio::sync::Mutex::new(rx)),
    )
});

/// Notified when setup is fully complete - either after a successful `IssueAcme` run or after
/// an explicit `FinishSetup` call.  The setup server waits on this before shutting down so the
/// main gRPC server can bind the same port afterward.
static SETUP_DONE_NOTIFY: LazyLock<Arc<Notify>> = LazyLock::new(|| Arc::new(Notify::new()));

const AUTH_HEADER: &str = "authorization";

pub(crate) struct ProxySetupServer {
    key_pair: Arc<Mutex<Option<defguard_certs::RcGenKeyPair>>>,
    logs_rx: LogsReceiver,
    current_session_token: Arc<Mutex<Option<String>>>,
    /// Sender used to request a graceful hand-off of port 80 from the main HTTP server loop
    /// before the ACME challenge listener binds.  `None` when the main server is not on port 80.
    port80_pause_tx: Option<mpsc::Sender<(oneshot::Sender<()>, oneshot::Receiver<()>)>>,
}

impl Clone for ProxySetupServer {
    fn clone(&self) -> Self {
        Self {
            key_pair: Arc::clone(&self.key_pair),
            logs_rx: Arc::clone(&self.logs_rx),
            current_session_token: Arc::clone(&self.current_session_token),
            port80_pause_tx: self.port80_pause_tx.clone(),
        }
    }
}

impl ProxySetupServer {
    pub fn new(
        logs_rx: LogsReceiver,
        port80_pause_tx: Option<mpsc::Sender<(oneshot::Sender<()>, oneshot::Receiver<()>)>>,
    ) -> Self {
        Self {
            key_pair: Arc::new(Mutex::new(None)),
            logs_rx,
            current_session_token: Arc::new(Mutex::new(None)),
            port80_pause_tx,
        }
    }

    /// Await setup connection from Defguard Core and process it.
    ///
    /// **Phase 1 - plain HTTP setup server:**
    /// Spins up a plain HTTP gRPC server on `addr` to handle the initial handshake: `Start`,
    /// `GetCsr`, `SendCert`.  The server shuts down as soon as `SendCert` deposits a
    /// `Configuration` into `SETUP_CHANNEL`.
    ///
    /// **Phase 2 - TLS gRPC server (same port):**
    /// Immediately after Phase 1 exits, a new TLS gRPC server is started on the same `addr`
    /// using the just-received cert+key.  Core reconnects over `https://` and calls either:
    /// - `IssueAcme` (Let's Encrypt flow): shuts down on successful ACME completion.
    /// - `FinishSetup` (non-ACME flows): shuts down immediately.
    ///
    /// On ACME failure the Phase-2 server stays alive so Core can retry without re-running
    /// the full adoption flow.
    ///
    /// Returns the received gRPC configuration (locally generated key pair and remotely signed
    /// certificate) upon successful setup.
    pub(crate) async fn await_initial_setup(
        &self,
        addr: SocketAddr,
    ) -> Result<Configuration, anyhow::Error> {
        info!("gRPC waiting for setup connection from Core on {addr}");

        let own_version = Version::parse(VERSION)?;
        debug!("Proxy version: {}", VERSION);

        let config_slot: Arc<tokio::sync::Mutex<Option<Configuration>>> =
            Arc::new(tokio::sync::Mutex::new(None));
        let config_slot_writer = Arc::clone(&config_slot);

        // Phase 1: plain HTTP
        Server::builder()
            .layer(tonic::service::InterceptorLayer::new(
                DefguardVersionInterceptor::new(
                    own_version.clone(),
                    DefguardComponent::Core,
                    MIN_CORE_VERSION,
                    false,
                ),
            ))
            .layer(DefguardVersionLayer::new(own_version.clone()))
            .add_service(proxy_setup_server::ProxySetupServer::new(self.clone()))
            .serve_with_shutdown(addr, async move {
                debug!("Phase 1: waiting for SendCert to deliver configuration");
                // SETUP_CHANNEL is CommsChannel<Option<Configuration>>, so recv() returns
                // Option<Option<Configuration>>.  send_cert always sends Some(cfg).
                if let Some(Some(cfg)) = SETUP_CHANNEL.1.lock().await.recv().await {
                    debug!("Phase 1: configuration received from SendCert");
                    *config_slot_writer.lock().await = Some(cfg);
                } else {
                    error!("Phase 1: SETUP_CHANNEL closed unexpectedly without configuration");
                }
                debug!("Phase 1: plain-HTTP server will now shut down");
            })
            .await
            .map_err(|err| {
                error!("Phase 1 gRPC server error: {err}");
                ApiError::Unexpected("Phase 1 gRPC server error during setup".into())
            })?;
        debug!("Phase 1: plain-HTTP setup server shut down on {addr}");

        let configuration = config_slot.lock().await.take().ok_or_else(|| {
            error!("No configuration received after Phase 1 setup");
            ApiError::Unexpected("No configuration received after Phase 1 setup".into())
        })?;

        // Phase 2: TLS gRPC server
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        debug!("Phase 2: starting TLS gRPC setup server on {addr}");

        let tls_config = ServerTlsConfig::new().identity(Identity::from_pem(
            configuration.grpc_cert_pem.as_bytes(),
            configuration.grpc_key_pem.as_bytes(),
        ));

        Server::builder()
            .tls_config(tls_config)
            .map_err(|err| {
                error!("Failed to configure TLS for Phase 2 setup server: {err}");
                ApiError::Unexpected("Failed to configure TLS for Phase 2 setup server".into())
            })?
            .layer(tonic::service::InterceptorLayer::new(
                DefguardVersionInterceptor::new(
                    own_version.clone(),
                    DefguardComponent::Core,
                    MIN_CORE_VERSION,
                    false,
                ),
            ))
            .layer(DefguardVersionLayer::new(own_version))
            .add_service(proxy_setup_server::ProxySetupServer::new(self.clone()))
            .serve_with_shutdown(addr, async {
                // Wait indefinitely for Core to either:
                //   - Call IssueAcme (Let's Encrypt): SETUP_DONE_NOTIFY fires on success.
                //   - Call FinishSetup (no ACME): SETUP_DONE_NOTIFY fires immediately.
                debug!("Phase 2: waiting for IssueAcme or FinishSetup signal");
                SETUP_DONE_NOTIFY.notified().await;
                debug!("Phase 2: setup done signal received; TLS server will shut down");
            })
            .await
            .map_err(|err| {
                error!("Phase 2 gRPC server error: {err}");
                ApiError::Unexpected("Phase 2 gRPC server error during setup".into())
            })?;
        debug!("Phase 2: TLS setup server shut down on {addr}");

        Ok(configuration)
    }

    fn is_setup_in_progress(&self) -> bool {
        let in_progress = self
            .current_session_token
            .lock()
            .expect("Failed to acquire lock on current session token during proxy setup")
            .is_some();
        debug!("Setup in progress check: {}", in_progress);
        in_progress
    }

    fn clear_setup_session(&self) {
        debug!("Terminating setup session");
        self.current_session_token
            .lock()
            .expect("Failed to acquire lock on current session token during proxy setup")
            .take();
        debug!("Setup session terminated");
    }

    fn initialize_setup_session(&self, token: String) {
        debug!("Establishing new setup session with Core");
        self.current_session_token
            .lock()
            .expect("Failed to acquire lock on current session token during proxy setup")
            .replace(token);
        debug!("Setup session established");
    }

    fn verify_session_token(&self, token: &str) -> bool {
        debug!("Validating setup session authorization");
        let is_valid = (*self
            .current_session_token
            .lock()
            .expect("Failed to acquire lock on current session token during proxy setup"))
        .as_ref()
        .is_some_and(|t| t == token);
        debug!("Authorization validation result: {}", is_valid);
        is_valid
    }
}

#[tonic::async_trait]
impl proxy_setup_server::ProxySetup for ProxySetupServer {
    type IssueAcmeStream = UnboundedReceiverStream<Result<AcmeIssueEvent, Status>>;
    type StartStream = UnboundedReceiverStream<Result<LogEntry, Status>>;

    #[instrument(skip(self, request))]
    async fn start(&self, request: Request<()>) -> Result<Response<Self::StartStream>, Status> {
        debug!("Core initiated setup process, preparing to stream logs");
        if self.is_setup_in_progress() {
            error!("Setup already in progress, rejecting new setup request");
            return Err(Status::resource_exhausted("Setup already in progress"));
        }

        debug!("Authenticating setup session with Core");
        let token = request
            .metadata()
            .get(AUTH_HEADER)
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.strip_prefix("Bearer "))
            .ok_or_else(|| Status::unauthenticated("Missing or invalid authorization token"))?;

        debug!("Setup session authenticated successfully");
        self.initialize_setup_session(token.to_string());

        debug!("Preparing to forward Proxy logs to Core in real-time");
        let logs_rx = self.logs_rx.clone();

        let (tx, rx) = mpsc::unbounded_channel();
        let self_clone = self.clone();
        debug!("Starting log streaming to Core");
        tokio::spawn(async move {
            loop {
                let maybe_log_entry = logs_rx.lock().await.try_recv();
                match maybe_log_entry {
                    Ok(log_entry) => {
                        if tx.send(Ok(log_entry)).is_err() {
                            debug!(
                                "Failed to send log entry to gRPC stream: receiver disconnected"
                            );
                            break;
                        }
                    }
                    Err(tokio::sync::mpsc::error::TryRecvError::Empty) => {
                        if tx.is_closed() {
                            debug!("gRPC stream receiver disconnected");
                            break;
                        }
                        tokio::task::yield_now().await;
                    }
                    Err(tokio::sync::mpsc::error::TryRecvError::Disconnected) => {
                        debug!("Logs receiver disconnected");
                        break;
                    }
                }
            }
            self_clone.clear_setup_session();
        });

        debug!("Log stream established, Core will now receive real-time Proxy logs");
        Ok(Response::new(UnboundedReceiverStream::new(rx)))
    }

    #[instrument(skip(self, request))]
    async fn get_csr(
        &self,
        request: Request<CertificateInfo>,
    ) -> Result<Response<DerPayload>, Status> {
        debug!("Core requested Certificate Signing Request (CSR) generation");
        let token = request
            .metadata()
            .get(AUTH_HEADER)
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.strip_prefix("Bearer "))
            .ok_or_else(|| Status::unauthenticated("Missing or invalid authorization token"))?;

        debug!("Validating Core's authorization for this setup step");
        if !self.verify_session_token(token) {
            error!("Invalid session token in get_csr request");
            return Err(Status::unauthenticated("Invalid session token"));
        }

        let setup_info = request.into_inner();
        debug!(
            "Will generate certificate for hostname: {}",
            setup_info.cert_hostname
        );

        debug!("Generating key pair");
        let key_pair = match defguard_certs::generate_key_pair() {
            Ok(kp) => kp,
            Err(err) => {
                error!("Failed to generate key pair: {err}");
                self.clear_setup_session();
                return Err(Status::internal("Failed to generate key pair"));
            }
        };
        debug!("Key pair created");

        let subject_alt_names = vec![setup_info.cert_hostname];
        debug!(
            "Preparing Certificate Signing Request for hostname: {:?}",
            subject_alt_names
        );

        let csr = match defguard_certs::Csr::new(
            &key_pair,
            &subject_alt_names,
            vec![
                // TODO: Change it?
                (defguard_certs::DnType::CommonName, "Defguard Proxy"),
                (defguard_certs::DnType::OrganizationName, "Defguard"),
            ],
        ) {
            Ok(csr) => csr,
            Err(err) => {
                error!("Failed to generate CSR: {err}");
                self.clear_setup_session();
                return Err(Status::internal(format!("Failed to generate CSR: {err}")));
            }
        };
        debug!("Certificate Signing Request prepared");

        self.key_pair
			.lock()
			.expect("Failed to acquire lock on key pair during proxy setup when trying to store generated key pair")
			.replace(key_pair);

        debug!("Encoding Certificate Signing Request for transmission");
        let csr_der = csr.to_der();
        let csr_request = DerPayload {
            der_data: csr_der.to_vec(),
        };
        debug!(
            "Sending Certificate Signing Request to Core for signing ({} bytes)",
            csr_request.der_data.len()
        );

        Ok(Response::new(csr_request))
    }

    #[instrument(skip(self, request))]
    async fn send_cert(&self, request: Request<DerPayload>) -> Result<Response<()>, Status> {
        debug!("Core sending back signed certificate for installation");
        let token = request
            .metadata()
            .get(AUTH_HEADER)
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.strip_prefix("Bearer "))
            .ok_or_else(|| Status::unauthenticated("Missing or invalid authorization token"))?;

        debug!("Validating Core's authorization to complete setup");
        if !self.verify_session_token(token) {
            error!("Invalid session token in send_cert request");
            return Err(Status::unauthenticated("Invalid session token"));
        }

        let der_payload = request.into_inner();
        let cert_der = der_payload.der_data;
        debug!(
            "Received signed certificate from Core ({} bytes)",
            cert_der.len()
        );

        debug!("Parsing received certificate DER data");
        let grpc_cert_pem =
            match defguard_certs::der_to_pem(&cert_der, defguard_certs::PemLabel::Certificate) {
                Ok(pem) => pem,
                Err(err) => {
                    error!("Failed to convert certificate DER to PEM: {err}");
                    self.clear_setup_session();
                    return Err(Status::internal(format!(
                        "Failed to convert certificate DER to PEM: {err}"
                    )));
                }
            };
        debug!("Certificate processed successfully");

        let key_pair = {
            let key_pair = self
				.key_pair
				.lock()
				.expect("Failed to acquire lock on key pair during proxy setup when trying to receive certificate")
				.take();
            if let Some(kp) = key_pair {
                kp
            } else {
                error!(
                    "Key pair not found during Proxy setup. Key pair generation step might have failed."
                );
                self.clear_setup_session();
                return Err(Status::internal(
                    "Key pair not found during Proxy setup. Key pair generation step might have failed.",
                ));
            }
        };

        let configuration = Configuration {
            grpc_key_pem: key_pair.serialize_pem(),
            grpc_cert_pem,
        };

        debug!("Passing configuration to gRPC server for finalization");
        match SETUP_CHANNEL.0.lock().await.send(Some(configuration)).await {
            Ok(()) => info!("Proxy configuration passed to gRPC server successfully"),
            Err(err) => {
                error!("Failed to send configuration to gRPC server: {err}");
                self.clear_setup_session();
                return Err(Status::internal(
                    "Failed to send configuration to gRPC server",
                ));
            }
        }

        self.clear_setup_session();
        debug!(
            "SendCert completed; Phase-1 session cleared, Phase-2 TLS server will accept new Start call"
        );

        debug!("Confirming successful setup to Core");
        Ok(Response::new(()))
    }

    #[instrument(skip(self, request))]
    async fn issue_acme(
        &self,
        request: Request<AcmeChallenge>,
    ) -> Result<Response<Self::IssueAcmeStream>, Status> {
        debug!("Core requested ACME HTTP-01 certificate issuance");
        let token = request
            .metadata()
            .get(AUTH_HEADER)
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.strip_prefix("Bearer "))
            .ok_or_else(|| Status::unauthenticated("Missing or invalid authorization token"))?;

        if !self.verify_session_token(token) {
            error!("Invalid session token in issue_acme request");
            return Err(Status::unauthenticated("Invalid session token"));
        }

        let challenge = request.into_inner();
        let domain = challenge.domain.clone();
        let use_staging = challenge.use_staging;
        let existing_credentials = challenge.account_credentials_json.clone();

        info!("Starting ACME HTTP-01 for domain: {domain} (staging={use_staging})");

        let (tx, rx) = mpsc::unbounded_channel::<Result<AcmeIssueEvent, Status>>();
        let self_clone = self.clone();

        // Emit the first progress step immediately - we are connected and about to start.
        let connecting_event = AcmeIssueEvent {
            payload: Some(acme_issue_event::Payload::Progress(AcmeProgress {
                step: AcmeStep::Connecting as i32,
            })),
        };
        let _ = tx.send(Ok(connecting_event));

        tokio::spawn(async move {
            // Request a graceful hand-off of port 80 from the main HTTP server if it is bound
            // there, so the ACME challenge listener can bind.  `port80_pause_tx` is `Some` only
            // when the main server runs on port 80; if it's `None` the port is already free.
            let permit: Option<Port80Permit> =
                if let Some(ref pause_tx) = self_clone.port80_pause_tx {
                    let (ready_tx, ready_rx) = oneshot::channel::<()>();
                    let (done_tx, done_rx) = oneshot::channel::<()>();
                    if pause_tx.send((ready_tx, done_rx)).await.is_err() {
                        error!(
                            "Failed to request port-80 hand-off for ACME setup; \
						 HTTP server may have stopped"
                        );
                        let _ = tx.send(Err(Status::internal(
                            "Failed to request port-80 hand-off for ACME",
                        )));
                        self_clone.clear_setup_session();
                        return;
                    }
                    Some(Port80Permit {
                        ready: ready_rx,
                        done_tx,
                    })
                } else {
                    // Main server is not on port 80 - no hand-off needed.
                    None
                };

            // Channel used by run_acme_http01 to emit intermediate progress steps.
            let (progress_tx, mut progress_rx) = mpsc::unbounded_channel::<AcmeStep>();

            // Forward progress steps from acme.rs onto the gRPC response stream.
            let tx_fwd = tx.clone();
            tokio::spawn(async move {
                while let Some(step) = progress_rx.recv().await {
                    let event = AcmeIssueEvent {
                        payload: Some(acme_issue_event::Payload::Progress(AcmeProgress {
                            step: step as i32,
                        })),
                    };
                    if tx_fwd.send(Ok(event)).is_err() {
                        // Core disconnected - stop forwarding.
                        break;
                    }
                }
            });

            let result = run_acme_http01(
                domain.clone(),
                use_staging,
                existing_credentials,
                permit,
                progress_tx,
            )
            .await;

            match result {
                Ok(acme_result) => {
                    let cert_event = AcmeIssueEvent {
                        payload: Some(acme_issue_event::Payload::Certificate(AcmeCertificate {
                            cert_pem: acme_result.cert_pem,
                            key_pem: acme_result.key_pem,
                            account_credentials_json: acme_result.account_credentials_json,
                        })),
                    };
                    if tx.send(Ok(cert_event)).is_err() {
                        error!(
                            "ACME result stream receiver disconnected before cert could be sent"
                        );
                    } else {
                        info!("ACME certificate for domain '{domain}' streamed to Core");
                    }
                    // Success: clear session and signal the setup server to shut down so the
                    // main gRPC server can start.
                    self_clone.clear_setup_session();
                    SETUP_DONE_NOTIFY.notify_one();
                    debug!("ACME success: setup server shutdown signaled");
                }
                Err(err) => {
                    error!("ACME HTTP-01 failed for domain '{domain}': {err}");
                    let _ = tx.send(Err(Status::internal(format!(
                        "ACME HTTP-01 certificate issuance failed: {err}"
                    ))));
                    // Failure: clear session only - do NOT notify SETUP_DONE_NOTIFY.
                    // The setup server stays alive so Core can retry (call Start + IssueAcme
                    // again) without needing a full re-adoption.
                    self_clone.clear_setup_session();
                    debug!("ACME failed: setup server remains alive for retry");
                }
            }
        });

        Ok(Response::new(UnboundedReceiverStream::new(rx)))
    }

    #[instrument(skip(self, request))]
    async fn finish_setup(&self, request: Request<()>) -> Result<Response<()>, Status> {
        debug!("Core signaled setup complete without ACME");
        let token = request
            .metadata()
            .get(AUTH_HEADER)
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.strip_prefix("Bearer "))
            .ok_or_else(|| Status::unauthenticated("Missing or invalid authorization token"))?;

        if !self.verify_session_token(token) {
            error!("Invalid session token in finish_setup request");
            return Err(Status::unauthenticated("Invalid session token"));
        }

        self.clear_setup_session();
        SETUP_DONE_NOTIFY.notify_one();
        info!("Setup finalized without ACME; setup server will shut down");

        Ok(Response::new(()))
    }
}
