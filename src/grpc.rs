use std::{
    any::Any,
    collections::HashMap,
    net::SocketAddr,
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc, LazyLock, Mutex,
    },
};

use defguard_version::{
    get_tracing_variables,
    server::{grpc::DefguardVersionInterceptor, DefguardVersionLayer},
    ComponentInfo, DefguardComponent, Version,
};
use tokio::sync::{mpsc, oneshot};
use tokio_stream::wrappers::UnboundedReceiverStream;
use tonic::{
    transport::{Identity, Server, ServerTlsConfig},
    Request, Response, Status, Streaming,
};
use tower::ServiceBuilder;
use tracing::Instrument;

use crate::{
    error::ApiError,
    http::GRPC_SERVER_RESTART_CHANNEL,
    proto::{
        core_request, core_response, proxy_server, proxy_setup_request, proxy_setup_response,
        CertResponse, CoreRequest, CoreResponse, CsrRequest, DeviceInfo, Done, ProxySetupRequest,
        ProxySetupResponse,
    },
    CommsChannel, MIN_CORE_VERSION, VERSION,
};

// connected clients
type ClientMap = HashMap<SocketAddr, mpsc::UnboundedSender<Result<CoreRequest, Status>>>;

static SETUP_CHANNEL: LazyLock<CommsChannel<Option<Configuration>>> = LazyLock::new(|| {
    let (tx, rx) = mpsc::channel(10);
    (
        Arc::new(tokio::sync::Mutex::new(tx)),
        Arc::new(tokio::sync::Mutex::new(rx)),
    )
});

#[derive(Debug, Clone, Default)]
pub(crate) struct Configuration {
    pub(crate) grpc_key_pem: String,
    pub(crate) grpc_cert_pem: String,
}

pub(crate) struct ProxyServer {
    current_id: Arc<AtomicU64>,
    clients: Arc<Mutex<ClientMap>>,
    results: Arc<Mutex<HashMap<u64, oneshot::Sender<core_response::Payload>>>>,
    pub(crate) connected: Arc<AtomicBool>,
    pub(crate) core_version: Arc<Mutex<Option<Version>>>,
    config: Arc<Mutex<Option<Configuration>>>,
    setup_in_progress: Arc<AtomicBool>,
}

impl ProxyServer {
    #[must_use]
    /// Create new `ProxyServer`.
    pub(crate) fn new() -> Self {
        Self {
            current_id: Arc::new(AtomicU64::new(1)),
            clients: Arc::new(Mutex::new(HashMap::new())),
            results: Arc::new(Mutex::new(HashMap::new())),
            connected: Arc::new(AtomicBool::new(false)),
            core_version: Arc::new(Mutex::new(None)),
            config: Arc::new(Mutex::new(None)),
            setup_in_progress: Arc::new(AtomicBool::new(false)),
        }
    }

    pub(crate) fn set_tls_config(&self, cert_pem: String, key_pem: String) -> Result<(), ApiError> {
        let mut lock = self.config.lock().unwrap();
        let config = lock.get_or_insert_with(Configuration::default);
        config.grpc_cert_pem = cert_pem;
        config.grpc_key_pem = key_pem;
        Ok(())
    }

    pub(crate) async fn await_setup(
        &self,
        addr: SocketAddr,
    ) -> Result<Configuration, anyhow::Error> {
        info!("gRPC waiting for setup connection from Core on {addr}");
        let server_builder = Server::builder();
        let mut server_config = None;

        let own_version = Version::parse(VERSION)?;

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
            .add_service(proxy_server::ProxyServer::new(self.clone()))
            .serve_with_shutdown(addr, async {
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

    pub(crate) fn configure(&self, config: Configuration) {
        let mut lock = self.config.lock().unwrap();
        *lock = Some(config);
    }

    pub(crate) fn get_configuration(&self) -> Option<Configuration> {
        let lock = self.config.lock().unwrap();
        lock.clone()
    }

    pub(crate) async fn run(self, addr: SocketAddr) -> Result<(), anyhow::Error> {
        info!("Starting gRPC server on {addr}");
        let config = self.get_configuration();
        let (grpc_cert, grpc_key) = if let Some(cfg) = config {
            (cfg.grpc_cert_pem, cfg.grpc_key_pem)
        } else {
            return Err(anyhow::anyhow!("gRPC server configuration is missing"));
        };

        let identity = Identity::from_pem(grpc_cert, grpc_key);
        let mut builder =
            Server::builder().tls_config(ServerTlsConfig::new().identity(identity))?;

        let own_version = Version::parse(VERSION)?;
        let versioned_service = ServiceBuilder::new()
            .layer(tonic::service::InterceptorLayer::new(
                DefguardVersionInterceptor::new(
                    own_version.clone(),
                    DefguardComponent::Core,
                    MIN_CORE_VERSION,
                    false,
                ),
            ))
            .layer(DefguardVersionLayer::new(own_version))
            .service(proxy_server::ProxyServer::new(self.clone()));

        builder
            .add_service(versioned_service)
            .serve_with_shutdown(addr, async move {
                let mut rx_lock = GRPC_SERVER_RESTART_CHANNEL.1.lock().await;
                rx_lock.recv().await;
                info!("Shutting down gRPC server for restart...");
            })
            .await
            .map_err(|err| {
                error!("gRPC server error: {err}");
                err
            })?;

        Ok(())
    }

    /// Sends message to the other side of RPC, with given `payload` and `device_info`.
    #[instrument(level = "debug", skip(self, payload))]
    pub(crate) fn send(
        &self,
        payload: core_request::Payload,
        device_info: DeviceInfo,
    ) -> Result<oneshot::Receiver<core_response::Payload>, ApiError> {
        if let Some(client_tx) = self.clients.lock().unwrap().values().next() {
            let id = self.current_id.fetch_add(1, Ordering::Relaxed);
            let res = CoreRequest {
                id,
                device_info: Some(device_info),
                payload: Some(payload),
            };
            if let Err(err) = client_tx.send(Ok(res)) {
                error!("Failed to send CoreRequest: {err}");
                return Err(ApiError::Unexpected("Failed to send CoreRequest".into()));
            }
            let (tx, rx) = oneshot::channel();
            self.results.lock().unwrap().insert(id, tx);
            self.connected.store(true, Ordering::Relaxed);
            Ok(rx)
        } else {
            error!("Defguard Core is not connected");
            self.connected.store(false, Ordering::Relaxed);
            Err(ApiError::Unexpected(
                "Defguard Core is not connected".into(),
            ))
        }
    }

    pub(crate) fn setup_completed(&self) -> bool {
        let lock = self.config.lock().unwrap();
        lock.is_some()
    }
}

impl Clone for ProxyServer {
    fn clone(&self) -> Self {
        Self {
            current_id: Arc::clone(&self.current_id),
            clients: Arc::clone(&self.clients),
            results: Arc::clone(&self.results),
            connected: Arc::clone(&self.connected),
            core_version: Arc::clone(&self.core_version),
            config: Arc::clone(&self.config),
            setup_in_progress: Arc::clone(&self.setup_in_progress),
        }
    }
}

#[tonic::async_trait]
impl proxy_server::Proxy for ProxyServer {
    type BidiStream = UnboundedReceiverStream<Result<CoreRequest, Status>>;
    type SetupStream = UnboundedReceiverStream<Result<ProxySetupRequest, Status>>;

    async fn setup(
        &self,
        request: Request<tonic::Streaming<ProxySetupResponse>>,
    ) -> Result<Response<Self::SetupStream>, Status> {
        if self.setup_in_progress.swap(true, Ordering::SeqCst) {
            info!("Proxy setup already in progress, rejecting concurrent setup attempt");
            return Err(Status::already_exists("Proxy setup is already in progress"));
        }

        info!("Starting proxy setup handshake");

        let (tx, rx) = mpsc::unbounded_channel();
        let tls_configured = self.setup_completed();
        let current_configuration = self.get_configuration();
        let setup_in_progress = Arc::clone(&self.setup_in_progress);

        tokio::spawn(
            async move {
                let mut stream = request.into_inner();
                let mut setup_successful = false;
                let initial_info = match stream.message().await {
                    Ok(Some(ProxySetupResponse {
                        payload: Some(proxy_setup_response::Payload::InitialSetupInfo(info)),
                    })) => {
                        info!("Received initial setup information from Defguard Core");
                        debug!("Initial setup info: {:?}", info);
                        info
                    }
                    Ok(Some(ProxySetupResponse {
                        payload: Some(proxy_setup_response::Payload::Done(Done {})),
                    })) => {
                        info!("Received setup termination message from Defguard Core, skipping setup phase");
                        setup_in_progress.store(false, Ordering::SeqCst);
                        return;
                    }
                    Ok(Some(msg)) => {
                        error!("Unexpected payload type in initial Proxy setup message: {:?}", msg);
                        let _ = tx.send(Err(Status::invalid_argument(
                            "Unexpected payload type in initial Proxy setup message",
                        )));
                        setup_in_progress.store(false, Ordering::SeqCst);
                        return;
                    }
                    Ok(None) => {
                        error!("No initial Proxy setup message received from Defguard Core");
                        let _ = tx.send(Err(Status::aborted(
                            "No initial Proxy setup message received from Defguard Core",
                        )));
                        setup_in_progress.store(false, Ordering::SeqCst);
                        return;
                    }
                    Err(err) => {
                        error!("Error receiving initial Proxy setup message from Defguard Core: {err}");
                        let _ = tx.send(Err(Status::internal(format!(
                            "Error receiving initial message: {err}"
                        ))));
                        setup_in_progress.store(false, Ordering::SeqCst);
                        return;
                    }
                };

                if tls_configured {
                    info!("Certificates already generated, skipping CSR generation");
                    if let Err(err) = tx.send(Ok(ProxySetupRequest {
                        payload: Some(proxy_setup_request::Payload::Done(Done {})),
                    })) {
                        error!("Failed to send Done message: {err}");
                    }
                    return;
                }

                info!("Generating new key pair and CSR for certificate issuance");
                let key_pair = match defguard_certs::generate_key_pair() {
                    Ok(kp) => kp,
                    Err(err) => {
                        error!("Failed to generate key pair: {err}");
                        let _ = tx.send(Err(Status::internal(format!(
                            "Failed to generate key pair: {err}"
                        ))));
                        setup_in_progress.store(false, Ordering::SeqCst);
                        return;
                    }
                };

                let subject_alt_names = vec![initial_info.cert_hostname];

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
                        let _ = tx.send(Err(Status::internal(format!(
                            "Failed to generate CSR: {err}"
                        ))));
                        setup_in_progress.store(false, Ordering::SeqCst);
                        return;
                    }
                };

                let csr_der = csr.to_der();
                let csr_request = CsrRequest {
                    csr_der: csr_der.to_vec(),
                };

                if let Err(err) = tx.send(Ok(ProxySetupRequest {
                    payload: Some(proxy_setup_request::Payload::CsrRequest(csr_request)),
                })) {
                    error!("Failed to send CsrRequest: {err}");
                    setup_in_progress.store(false, Ordering::SeqCst);
                    return;
                }

                debug!("Sent CSR request to Core");

                let mut configuration = current_configuration;

                loop {
                    match stream.message().await {
                        Ok(Some(response)) => match response.payload {
                            Some(proxy_setup_response::Payload::CertResponse(CertResponse {
                                cert_der,
                            })) => {
                                debug!("Received CertResponse from Defguard Core");
                                let grpc_cert_pem = match defguard_certs::der_to_pem(
                                    &cert_der,
                                    defguard_certs::PemLabel::Certificate,
                                ) {
                                    Ok(pem) => pem,
                                    Err(err) => {
                                        error!("Failed to convert certificate DER to PEM: {err}");
                                        let _ = tx.send(Err(Status::internal(format!(
                                            "Failed to convert certificate DER to PEM: {err}"
                                        ))));
                                        setup_in_progress.store(false, Ordering::SeqCst);
                                        return;
                                    }
                                };

                                configuration = Some(Configuration {
                                    grpc_key_pem: key_pair.serialize_pem(),
                                    grpc_cert_pem,
                                });

                                if let Err(err) = tx.send(Ok(ProxySetupRequest {
                                    payload: Some(proxy_setup_request::Payload::Done(Done {})),
                                })) {
                                    error!("Failed to send Done message: {err}");
                                    // Can't send error since tx already failed
                                    setup_in_progress.store(false, Ordering::SeqCst);
                                    return;
                                }

                                debug!("Sent Done message to Defguard Core");
                            }
                            Some(proxy_setup_response::Payload::Done(Done {})) => {
                                debug!("Received setup termination message from Defguard Core");
                                let lock = SETUP_CHANNEL.0.lock().await;
                                debug!("Passing Proxy configuration to the gRPC server");
                                if let Some(configuration) = configuration.take() {
                                    if let Err(err) = lock.send(Some(configuration)).await {
                                        error!("Failed to pass the received Proxy configuration to the gRPC server: {err:?}");
                                    } else {
                                        debug!("Passed Proxy configuration to the gRPC server");
                                        setup_successful = true;
                                    }
                                } else {
                                    error!(
                                        "No configuration available to pass to the gRPC server on setup completion"
                                    );
                                }

                                info!("Setup handshake completed successfully");
                                break;
                            }
                            _ => {
                                error!(
                                    "Unexpected payload type while waiting for setup message: {:?}",
                                    response.payload
                                );
                                let _ = tx.send(Err(Status::invalid_argument(
                                    "Unexpected payload type while waiting for setup message",
                                )));
                                setup_in_progress.store(false, Ordering::SeqCst);
                                return;
                            }
                        },
                        Ok(None) => {
                            error!("gRPC setup stream has been closed unexpectedly");
                            let _ = tx.send(Err(Status::aborted(
                                "gRPC stream has been closed unexpectedly",
                            )));
                            setup_in_progress.store(false, Ordering::SeqCst);
                            return;
                        }
                        Err(err) => {
                            error!("gRPC setup client error while waiting for CertResponse: {err}");
                            let _ =
                                tx.send(Err(Status::internal(format!("gRPC client error: {err}"))));
                            setup_in_progress.store(false, Ordering::SeqCst);
                            return;
                        }
                    }
                }

                setup_in_progress.store(false, Ordering::SeqCst);
                if setup_successful {
                    info!("Proxy setup completed successfully");
                } else {
                    warn!("Proxy setup did not complete successfully");
                }
            }
            .instrument(tracing::Span::current()),
        );

        Ok(Response::new(UnboundedReceiverStream::new(rx)))
    }

    /// Handle bidirectional communication with Defguard core.
    #[instrument(name = "bidirectional_communication", level = "info", skip(self))]
    async fn bidi(
        &self,
        request: Request<Streaming<CoreResponse>>,
    ) -> Result<Response<Self::BidiStream>, Status> {
        if !self.setup_completed() {
            error!("Received bidi connection before setup completion");
            return Err(Status::failed_precondition(
                "Setup must be completed before establishing bidi connection",
            ));
        }

        let Some(address) = request.remote_addr() else {
            error!("Failed to determine client address for request: {request:?}");
            return Err(Status::internal("Failed to determine client address"));
        };
        let maybe_info = ComponentInfo::from_metadata(request.metadata());
        let (version, info) = get_tracing_variables(&maybe_info);
        *self.core_version.lock().unwrap() = Some(version.clone());

        let span = tracing::info_span!("core_bidi_stream", component = %DefguardComponent::Core,
            version = version.to_string(), info);
        let _guard = span.enter();

        info!("Defguard Core gRPC client connected from: {address}");

        let (tx, rx) = mpsc::unbounded_channel();
        self.clients.lock().unwrap().insert(address, tx);
        self.connected.store(true, Ordering::Relaxed);

        let clients = Arc::clone(&self.clients);
        let results = Arc::clone(&self.results);
        let connected = Arc::clone(&self.connected);
        let mut stream = request.into_inner();
        tokio::spawn(
            async move {
                loop {
                    match stream.message().await {
                        Ok(Some(response)) => {
                            debug!("Received message from Defguard Core ID={}", response.id);
                            connected.store(true, Ordering::Relaxed);
                            if let Some(payload) = response.payload {
                                let maybe_rx = results.lock().unwrap().remove(&response.id);
                                if let Some(rx) = maybe_rx {
                                    if let Err(err) = rx.send(payload) {
                                        error!("Failed to send message to rx {:?}", err.type_id());
                                    }
                                } else {
                                    error!("Missing receiver for response #{}", response.id);
                                }
                            }
                        }
                        Ok(None) => {
                            info!("gRPC stream has been closed");
                            break;
                        }
                        Err(err) => {
                            error!("gRPC client error: {err}");
                            break;
                        }
                    }
                }
                info!("Defguard core client disconnected: {address}");
                connected.store(false, Ordering::Relaxed);
                clients.lock().unwrap().remove(&address);
            }
            .instrument(tracing::Span::current()),
        );

        Ok(Response::new(UnboundedReceiverStream::new(rx)))
    }
}
