use std::{
    net::SocketAddr,
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use defguard_version::{
    DefguardComponent, Version,
    server::{DefguardVersionLayer, grpc::DefguardVersionInterceptor},
};
use rustls_pki_types::{CertificateDer, UnixTime};
use tokio::{
    fs::OpenOptions,
    io::AsyncWriteExt,
    sync::{mpsc, oneshot},
};
use tokio_stream::wrappers::UnboundedReceiverStream;
use tonic::{Request, Response, Status, service::InterceptorLayer, transport::Server};
use webpki::{KeyUsage, anchor_from_trusted_cert};

use crate::{
    LogsReceiver, MIN_CORE_VERSION, VERSION,
    config::EnvConfig,
    error::ApiError,
    grpc::TlsConfig,
    proto::{CertBundle, CertificateInfo, DerPayload, LogEntry, proxy_setup_server},
};

const AUTH_HEADER: &str = "authorization";

pub const GRPC_CERT_NAME: &str = "proxy_grpc_cert.pem";
pub const GRPC_KEY_NAME: &str = "proxy_grpc_key.pem";
pub const GRPC_CA_CERT_NAME: &str = "grpc_ca_cert.pem";
pub const CORE_CLIENT_CERT_NAME: &str = "core_client_cert.pem";

/// Verify that both `component_der` and `core_client_der` are signed by `ca_der`.
///
/// Uses ECDSA P-256 via `aws-lc-rs` (Linux-only deployment; FIPS-capable).
/// The gateway counterpart uses `ring` for FreeBSD/OPNsense compatibility.
/// Both verify the same algorithm set: `ECDSA_P256_SHA256` and `ECDSA_P256_SHA384`.
/// Returns an error message string on any failure so the caller can forward it as a gRPC status.
fn validate_cert_bundle(
    ca_der: &[u8],
    component_der: &[u8],
    core_client_der: &[u8],
) -> Result<(), String> {
    let sig_algs: &[&dyn rustls_pki_types::SignatureVerificationAlgorithm] = &[
        webpki::aws_lc_rs::ECDSA_P256_SHA256,
        webpki::aws_lc_rs::ECDSA_P256_SHA384,
    ];

    let ca_cert_der = CertificateDer::from(ca_der);
    let trust_anchor = anchor_from_trusted_cert(&ca_cert_der)
        .map_err(|e| format!("Failed to parse CA certificate as trust anchor: {e}"))?;
    let trust_anchors = [trust_anchor];

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::ZERO);
    let time = UnixTime::since_unix_epoch(now);

    // Verify component (server) certificate.
    let component_cert_der = CertificateDer::from(component_der);
    let component_ee = webpki::EndEntityCert::try_from(&component_cert_der)
        .map_err(|e| format!("Failed to parse component certificate: {e}"))?;
    component_ee
        .verify_for_usage(
            sig_algs,
            &trust_anchors,
            &[],
            time,
            KeyUsage::server_auth(),
            None,
            None,
        )
        .map_err(|e| format!("Component certificate failed chain validation: {e}"))?;

    // Verify Core client certificate.
    let core_client_cert_der = CertificateDer::from(core_client_der);
    let core_client_ee = webpki::EndEntityCert::try_from(&core_client_cert_der)
        .map_err(|e| format!("Failed to parse Core client certificate: {e}"))?;
    core_client_ee
        .verify_for_usage(
            sig_algs,
            &trust_anchors,
            &[],
            time,
            KeyUsage::client_auth(),
            None,
            None,
        )
        .map_err(|e| format!("Core client certificate failed chain validation: {e}"))?;

    Ok(())
}

async fn save_tls_certs(tls_config: &TlsConfig, cert_dir: &Path) -> Result<(), String> {
    let cert_path = cert_dir.join(GRPC_CERT_NAME);
    let key_path = cert_dir.join(GRPC_KEY_NAME);
    let ca_cert_path = cert_dir.join(GRPC_CA_CERT_NAME);
    let client_cert_path = cert_dir.join(CORE_CLIENT_CERT_NAME);

    let mut options = OpenOptions::new();
    options.write(true).create(true).truncate(true);
    #[cfg(unix)]
    options.mode(0o600); // rw-------

    // PEM-encode the Core client certificate DER for serial pinning on restart.
    let core_client_cert_pem = defguard_certs::der_to_pem(
        &tls_config.core_client_cert_der,
        defguard_certs::PemLabel::Certificate,
    )
    .map_err(|err| format!("Failed to PEM-encode Core client certificate: {err}"))?;

    // Write component (server) certificate.
    options
        .clone()
        .open(&cert_path)
        .await
        .map_err(|err| {
            format!(
                "Cannot open certificate file {}: {err}",
                cert_path.display()
            )
        })?
        .write_all(tls_config.grpc_cert_pem.as_bytes())
        .await
        .map_err(|err| {
            format!(
                "Cannot write certificate file {}: {err}",
                cert_path.display()
            )
        })?;

    // Write private key.
    options
        .clone()
        .open(&key_path)
        .await
        .map_err(|err| format!("Cannot open key file {}: {err}", key_path.display()))?
        .write_all(tls_config.grpc_key_pem.as_bytes())
        .await
        .map_err(|err| format!("Cannot write key file {}: {err}", key_path.display()))?;

    // Write CA certificate.
    options
        .clone()
        .open(&ca_cert_path)
        .await
        .map_err(|err| {
            format!(
                "Cannot open CA certificate file {}: {err}",
                ca_cert_path.display()
            )
        })?
        .write_all(tls_config.grpc_ca_cert_pem.as_bytes())
        .await
        .map_err(|err| {
            format!(
                "Cannot write CA certificate file {}: {err}",
                ca_cert_path.display()
            )
        })?;

    // Write Core client certificate (PEM) for serial pinning on restart.
    options
        .open(&client_cert_path)
        .await
        .map_err(|err| {
            format!(
                "Cannot open Core client certificate file {}: {err}",
                client_cert_path.display()
            )
        })?
        .write_all(core_client_cert_pem.as_bytes())
        .await
        .map_err(|err| {
            format!(
                "Cannot write Core client certificate file {}: {err}",
                client_cert_path.display()
            )
        })?;

    Ok(())
}

pub(crate) struct ProxySetupServer {
    key_pair: Arc<Mutex<Option<defguard_certs::RcGenKeyPair>>>,
    logs_rx: LogsReceiver,
    current_session_token: Arc<Mutex<Option<String>>>,
    setup_tx: Arc<tokio::sync::Mutex<Option<oneshot::Sender<TlsConfig>>>>,
    setup_rx: Arc<tokio::sync::Mutex<oneshot::Receiver<TlsConfig>>>,
    adoption_expired: Arc<AtomicBool>,
    cert_dir: Arc<PathBuf>,
}

impl Clone for ProxySetupServer {
    fn clone(&self) -> Self {
        Self {
            key_pair: Arc::clone(&self.key_pair),
            logs_rx: Arc::clone(&self.logs_rx),
            current_session_token: Arc::clone(&self.current_session_token),
            setup_tx: Arc::clone(&self.setup_tx),
            setup_rx: Arc::clone(&self.setup_rx),
            adoption_expired: Arc::clone(&self.adoption_expired),
            cert_dir: Arc::clone(&self.cert_dir),
        }
    }
}

impl ProxySetupServer {
    pub fn new(logs_rx: LogsReceiver, cert_dir: PathBuf) -> Self {
        let (setup_tx, setup_rx) = oneshot::channel();
        Self {
            key_pair: Arc::new(Mutex::new(None)),
            logs_rx,
            current_session_token: Arc::new(Mutex::new(None)),
            setup_tx: Arc::new(tokio::sync::Mutex::new(Some(setup_tx))),
            setup_rx: Arc::new(tokio::sync::Mutex::new(setup_rx)),
            adoption_expired: Arc::new(AtomicBool::new(false)),
            cert_dir: Arc::new(cert_dir),
        }
    }

    /// Await setup connection from Defguard Core and process it.
    ///
    /// Spins up a plain HTTP gRPC server on `addr` to handle the initial handshake: `Start`,
    /// `GetCsr`, `SendCert`.  The server shuts down as soon as `SendCert` delivers a
    /// `TlsConfig` through the oneshot channel, after which this function returns the received
    /// gRPC configuration (locally generated key pair and remotely signed certificate).
    ///
    /// A timeout is started in the background using `config.adoption_timeout()`. If the timeout
    /// elapses before setup completes, the `adoption_expired` flag is set and incoming `Start`
    /// requests are rejected with `failed_precondition` until the Edge is restarted.
    /// On successful adoption the timeout is cancelled.
    pub(crate) async fn await_initial_setup(
        &self,
        addr: SocketAddr,
        config: &EnvConfig,
    ) -> Result<TlsConfig, anyhow::Error> {
        let adoption_timeout = config.adoption_timeout();
        info!(
            "gRPC waiting for setup connection from Core on {addr} for {} min",
            adoption_timeout.as_secs() / 60
        );

        let adoption_expired = Arc::clone(&self.adoption_expired);
        let (cancel_tx, cancel_rx) = oneshot::channel::<()>();
        tokio::spawn(async move {
            tokio::select! {
                () = tokio::time::sleep(adoption_timeout) => {
                    adoption_expired.store(true, Ordering::Relaxed);
                    error!(
                        "Edge adoption expired and is now blocked. Restart the Edge to enable adoption."
                    );
                }
                _ = cancel_rx => {}
            }
        });
        let own_version = Version::parse(VERSION)?;
        debug!("Edge version: {}", VERSION);

        let setup_rx = Arc::clone(&self.setup_rx);
        let mut server_config = None;

        Server::builder()
            .layer(InterceptorLayer::new(DefguardVersionInterceptor::new(
                own_version.clone(),
                DefguardComponent::Core,
                MIN_CORE_VERSION,
                false,
            )))
            .layer(DefguardVersionLayer::new(own_version.clone()))
            .add_service(proxy_setup_server::ProxySetupServer::new(self.clone()))
            .serve_with_shutdown(addr, async {
                debug!("Waiting for SendCert to deliver configuration");
                let mut rx_guard = setup_rx.lock().await;
                match (&mut *rx_guard).await {
                    Ok(cfg) => {
                        debug!("Configuration received from SendCert");
                        server_config = Some(cfg);
                    }
                    Err(err) => {
                        error!("Setup communication channel closed unexpectedly: {err}");
                    }
                }
                debug!("Plain-HTTP server will now shut down");
            })
            .await
            .map_err(|err| {
                error!("Setup gRPC server error: {err}");
                ApiError::Unexpected("gRPC server error during setup".into())
            })?;
        debug!("Plain-HTTP setup server shut down on {addr}");

        let tls_config = server_config.ok_or_else(|| {
            error!("No configuration received after setup");
            ApiError::Unexpected("No configuration received after setup".into())
        })?;

        // Skip blocking Edge adoption if adoption was already done
        let _ = cancel_tx.send(());

        Ok(tls_config)
    }

    fn is_setup_in_progress(&self) -> bool {
        let in_progress = self
            .current_session_token
            .lock()
            .expect("Failed to acquire lock on current session token during Edge setup")
            .is_some();
        debug!("Setup in progress check: {}", in_progress);
        in_progress
    }

    fn clear_setup_session(&self) {
        debug!("Terminating setup session");
        self.current_session_token
            .lock()
            .expect("Failed to acquire lock on current session token during Edge setup")
            .take();
        debug!("Setup session terminated");
    }

    fn initialize_setup_session(&self, token: String) {
        debug!("Establishing new setup session with Core");
        self.current_session_token
            .lock()
            .expect("Failed to acquire lock on current session token during Edge setup")
            .replace(token);
        debug!("Setup session established");
    }

    fn verify_session_token(&self, token: &str) -> bool {
        debug!("Validating setup session authorization");
        let is_valid = (*self
            .current_session_token
            .lock()
            .expect("Failed to acquire lock on current session token during Edge setup"))
        .as_ref()
        .is_some_and(|t| t == token);
        debug!("Authorization validation result: {}", is_valid);
        is_valid
    }
}

#[tonic::async_trait]
impl proxy_setup_server::ProxySetup for ProxySetupServer {
    type StartStream = UnboundedReceiverStream<Result<LogEntry, Status>>;

    #[instrument(skip(self, request))]
    async fn start(&self, request: Request<()>) -> Result<Response<Self::StartStream>, Status> {
        debug!("Core initiated setup process, preparing to stream logs");
        if self.adoption_expired.load(Ordering::Relaxed) {
            let error_message =
                "Edge adoption expired and is now blocked. Restart the Edge to enable adoption.";
            error!("{error_message}");
            return Err(Status::failed_precondition(error_message));
        }
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

        debug!("Preparing to forward Edge logs to Core in real-time");
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

        debug!("Log stream established, Core will now receive real-time Edge logs");
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
			.expect("Failed to acquire lock on key pair during Edge setup when trying to store generated key pair")
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
    async fn send_cert(&self, request: Request<CertBundle>) -> Result<Response<()>, Status> {
        debug!("Core sending back signed certificate bundle for installation");
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

        let bundle = request.into_inner();

        debug!("Validating certificate bundle received from Core");
        if let Err(reason) = validate_cert_bundle(
            &bundle.ca_cert_der,
            &bundle.component_cert_der,
            &bundle.core_client_cert_der,
        ) {
            error!("Certificate bundle validation failed: {reason}");
            self.clear_setup_session();
            return Err(Status::invalid_argument(reason));
        }
        debug!("Certificate bundle validated successfully against CA");

        debug!(
            "Received component certificate from Core ({} bytes)",
            bundle.component_cert_der.len()
        );
        debug!("Parsing component certificate DER data");
        let grpc_cert_pem = match defguard_certs::der_to_pem(
            &bundle.component_cert_der,
            defguard_certs::PemLabel::Certificate,
        ) {
            Ok(pem) => pem,
            Err(err) => {
                error!("Failed to convert component certificate DER to PEM: {err}");
                self.clear_setup_session();
                return Err(Status::internal(format!(
                    "Failed to convert component certificate DER to PEM: {err}"
                )));
            }
        };

        debug!(
            "Received CA certificate from Core ({} bytes)",
            bundle.ca_cert_der.len()
        );
        debug!("Parsing CA certificate DER data");
        let grpc_ca_cert_pem = match defguard_certs::der_to_pem(
            &bundle.ca_cert_der,
            defguard_certs::PemLabel::Certificate,
        ) {
            Ok(pem) => pem,
            Err(err) => {
                error!("Failed to convert CA certificate DER to PEM: {err}");
                self.clear_setup_session();
                return Err(Status::internal(format!(
                    "Failed to convert CA certificate DER to PEM: {err}"
                )));
            }
        };

        debug!(
            "Received Core client certificate ({} bytes); will pin serial for mTLS",
            bundle.core_client_cert_der.len()
        );

        let key_pair = {
            let key_pair = self
				.key_pair
				.lock()
				.expect("Failed to acquire lock on key pair during Edge setup when trying to receive certificate")
				.take();
            if let Some(kp) = key_pair {
                kp
            } else {
                error!(
                    "Key pair not found during Edge setup. Key pair generation step might have failed."
                );
                self.clear_setup_session();
                return Err(Status::internal(
                    "Key pair not found during Edge setup. Key pair generation step might have failed.",
                ));
            }
        };

        let configuration = TlsConfig {
            grpc_key_pem: key_pair.serialize_pem(),
            grpc_cert_pem,
            grpc_ca_cert_pem,
            core_client_cert_der: bundle.core_client_cert_der,
        };

        debug!("Saving TLS certificate files to disk");
        if let Err(err) = save_tls_certs(&configuration, &self.cert_dir).await {
            error!("Failed to save TLS certificates: {err}");
            self.clear_setup_session();
            return Err(Status::internal(format!(
                "Failed to save TLS certificates: {err}"
            )));
        }
        debug!("TLS certificate files saved successfully");

        debug!("Passing configuration to gRPC server for finalization");
        let Some(sender) = self.setup_tx.lock().await.take() else {
            error!("Setup channel sender already consumed");
            self.clear_setup_session();
            return Err(Status::internal("Setup already completed"));
        };
        sender.send(configuration).map_err(|_| {
            let msg = "Failed to send setup configuration through channel";
            error!(msg);
            self.clear_setup_session();
            Status::internal(msg)
        })?;
        info!("Edge configuration passed to gRPC server successfully");

        self.clear_setup_session();
        debug!("SendCert completed; setup session cleared");

        debug!("Confirming successful setup to Core");
        Ok(Response::new(()))
    }
}
