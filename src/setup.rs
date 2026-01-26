use std::{
    net::SocketAddr,
    sync::{Arc, LazyLock, Mutex},
};

use defguard_version::{
    server::{grpc::DefguardVersionInterceptor, DefguardVersionLayer},
    DefguardComponent, Version,
};
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;
use tonic::{transport::Server, Request, Response, Status};

use crate::{
    error::ApiError,
    grpc::Configuration,
    proto::{proxy_setup_server, CertificateInfo, DerPayload, LogEntry},
    CommsChannel, LogsReceiver, MIN_CORE_VERSION, VERSION,
};

static SETUP_CHANNEL: LazyLock<CommsChannel<Option<Configuration>>> = LazyLock::new(|| {
    let (tx, rx) = mpsc::channel(10);
    (
        Arc::new(tokio::sync::Mutex::new(tx)),
        Arc::new(tokio::sync::Mutex::new(rx)),
    )
});

const AUTH_HEADER: &str = "authorization";

pub(crate) struct ProxySetupServer {
    key_pair: Arc<Mutex<Option<defguard_certs::RcGenKeyPair>>>,
    logs_rx: LogsReceiver,
    current_session_token: Arc<Mutex<Option<String>>>,
}

impl Clone for ProxySetupServer {
    fn clone(&self) -> Self {
        Self {
            key_pair: Arc::clone(&self.key_pair),
            logs_rx: Arc::clone(&self.logs_rx),
            current_session_token: Arc::clone(&self.current_session_token),
        }
    }
}

impl ProxySetupServer {
    pub fn new(logs_rx: LogsReceiver) -> Self {
        Self {
            key_pair: Arc::new(Mutex::new(None)),
            logs_rx,
            current_session_token: Arc::new(Mutex::new(None)),
        }
    }

    /// Await setup connection from Defguard Core and process it.
    /// Spins up a dedicated temporary gRPC server over HTTP to handle the setup process.
    /// The setup process involves generating a CSR, sending it to Core and receiving signed TLS certificates.
    /// Returns the received gRPC configuration (locally generated key pair and remotely signed certificate) upon successful setup.
    pub(crate) async fn await_initial_setup(
        &self,
        addr: SocketAddr,
    ) -> Result<Configuration, anyhow::Error> {
        info!("gRPC waiting for setup connection from Core on {addr}");
        debug!("Initializing gRPC server builder for setup process");
        let server_builder = Server::builder();
        let mut server_config = None;

        debug!("Parsing proxy version: {}", VERSION);
        let own_version = Version::parse(VERSION)?;

        debug!(
            "Setting up version interceptor with minimum Core version: {}",
            MIN_CORE_VERSION
        );
        server_builder
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
                debug!("Setup server started, waiting for configuration from Core");
                let config = SETUP_CHANNEL.1.lock().await.recv().await;
                if let Some(cfg) = config {
                    debug!("Received the passed Proxy configuration");
                    server_config = cfg;
                } else {
                    error!("Setup communication channel closed unexpectedly");
                }
            })
            .await
            .map_err(|err| {
                error!("gRPC server error during setup: {err}");
                ApiError::Unexpected("gRPC server error during setup".into())
            })?;

        debug!("gRPC setup server on {addr} has shutdown after completing setup");
        debug!("Validating received server configuration");

        Ok(server_config.map_or_else(
            || {
                error!("No server configuration received after setup completion");
                Err(ApiError::Unexpected(
                    "No server configuration received after setup".into(),
                ))
            },
            |cfg| {
                debug!("Returning received server configuration");
                Ok(cfg)
            },
        )?)
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
                error!("Key pair not found during Proxy setup. Key pair generation step might have failed.");
                self.clear_setup_session();
                return Err(Status::internal("Key pair not found during Proxy setup. Key pair generation step might have failed."));
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

        debug!("Setup process completed successfully, cleaning up temporary session");
        self.clear_setup_session();

        debug!("Confirming successful setup to Core");
        Ok(Response::new(()))
    }
}
