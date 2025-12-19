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

pub(crate) const GRPC_CERTS_PATH: &str = "/Volumes/dysk/praca/proxy/proxy_certs/";

static SETUP_CHANNEL: LazyLock<CommsChannel<Option<Configuration>>> = LazyLock::new(|| {
    let (tx, rx) = mpsc::channel(10);
    (
        Arc::new(tokio::sync::Mutex::new(tx)),
        Arc::new(tokio::sync::Mutex::new(rx)),
    )
});

#[derive(Debug, Clone, Default)]
pub(crate) struct Configuration {
    pub(crate) grpc_key_pem: Option<String>,
    pub(crate) grpc_cert_pem: Option<String>,
}

pub(crate) struct ProxyServer {
    current_id: Arc<AtomicU64>,
    clients: Arc<Mutex<ClientMap>>,
    results: Arc<Mutex<HashMap<u64, oneshot::Sender<core_response::Payload>>>>,
    pub(crate) connected: Arc<AtomicBool>,
    pub(crate) core_version: Arc<Mutex<Option<Version>>>,
    config: Arc<Mutex<Option<Configuration>>>,
    setup_complete: Arc<AtomicBool>,
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
            setup_complete: Arc::new(AtomicBool::new(false)),
        }
    }

    pub(crate) async fn set_tls_config(
        &self,
        cert_pem: String,
        key_pem: String,
    ) -> Result<(), ApiError> {
        let mut lock = self.config.lock().unwrap();
        let config = lock.get_or_insert_with(Configuration::default);
        config.grpc_cert_pem = Some(cert_pem);
        config.grpc_key_pem = Some(key_pem);
        Ok(())
    }

    pub(crate) async fn await_setup(&self, addr: SocketAddr) -> Result<Configuration, ApiError> {
        info!("gRPC waiting for setup connection from Core on {addr}");
        let mut server_builder = Server::builder();
        let mut server_config = None;

        server_builder
            .add_service(proxy_server::ProxyServer::new(self.clone()))
            .serve_with_shutdown(addr, async {
                let config = SETUP_CHANNEL.1.lock().await.recv().await;
                if let Some(cfg) = config {
                    info!("Received setup configuration: {:?}", cfg);
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

        info!("gRPC setup server on {addr} has shut down");

        server_config.map_or_else(
            || {
                error!("No server configuration received after setup completion");
                Err(ApiError::Unexpected(
                    "No server configuration received after setup".into(),
                ))
            },
            |cfg| {
                info!("gRPC setup handshake completed.");
                Ok(cfg)
            },
        )
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
            (None, None)
        };

        info!("Configured gRPC certificate: {grpc_cert:?}");
        info!("Configured gRPC key: {grpc_key:?}");

        let mut builder = if let (Some(cert), Some(key)) = (grpc_cert, grpc_key) {
            let identity = Identity::from_pem(cert, key);
            Server::builder().tls_config(ServerTlsConfig::new().identity(identity))?
        } else {
            Server::builder()
        };
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

    fn setup_completed(&self) -> bool {
        let lock = self.config.lock().unwrap();
        lock.is_some()
    }

    fn tls_configured(&self) -> bool {
        let lock = self.config.lock().unwrap();
        if let Some(cfg) = &*lock {
            cfg.grpc_cert_pem.is_some() && cfg.grpc_key_pem.is_some()
        } else {
            false
        }
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
            setup_complete: Arc::clone(&self.setup_complete),
        }
    }
}

#[tonic::async_trait]
impl proxy_server::Proxy for ProxyServer {
    type BidiStream = UnboundedReceiverStream<Result<CoreRequest, Status>>;
    type ProxySetupStream = UnboundedReceiverStream<Result<ProxySetupRequest, Status>>;

    async fn proxy_setup(
        &self,
        request: Request<tonic::Streaming<ProxySetupResponse>>,
    ) -> Result<Response<Self::ProxySetupStream>, Status> {
        info!("Starting proxy setup handshake");

        let (tx, rx) = mpsc::unbounded_channel();
        let tls_configured = self.tls_configured();
        let current_configuration = self.get_configuration();

        tokio::spawn(
            async move {
                let mut stream = request.into_inner();
                let initial_info =
                    if let Some(initial_message) = stream.message().await.unwrap_or(None) {
                        info!("Received initial connection from Defguard Core");
                        if let Some(proxy_setup_response::Payload::InitialSetupInfo(info)) =
                            initial_message.payload
                        {
                            info!("Initial message confirmed from Defguard Core: {:?}", info);
                            info
                        } else {
                            error!("Unexpected payload type in initial message");
                            return;
                        }
                    } else {
                        error!("No initial message received from Defguard Core");
                        return;
                    };

                info!(
                    "Received initial connection from Defguard Core: {:?}",
                    initial_info
                );

                let mut key_pair = None;

                if tls_configured {
                    info!("TLS is already configured, skipping CSR generation");
                    if let Err(err) = tx.send(Ok(ProxySetupRequest {
                        payload: Some(proxy_setup_request::Payload::Done(Done {})),
                    })) {
                        error!("Failed to send Done message: {err}");
                    }
                } else {
                    let new_key_pair = match defguard_certs::generate_key_pair() {
                        Ok(kp) => kp,
                        Err(err) => {
                            error!("Failed to generate key pair: {err}");
                            return;
                        }
                    };

                    let subject_alt_names = vec![initial_info.cert_hostname];

                    let csr = match defguard_certs::Csr::new(
                        &new_key_pair,
                        &subject_alt_names,
                        vec![
                            (defguard_certs::DnType::CommonName, "Defguard Proxy"),
                            (defguard_certs::DnType::OrganizationName, "Defguard"),
                        ],
                    ) {
                        Ok(csr) => csr,
                        Err(err) => {
                            error!("Failed to generate CSR: {err}");
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
                        return;
                    }

                    info!("Sent CSR request to Defguard Core");

                    key_pair = Some(new_key_pair);
                }

                let mut configuration = current_configuration;

                loop {
                    match stream.message().await {
                        Ok(Some(response)) => match response.payload {
                            Some(proxy_setup_response::Payload::CertResponse(CertResponse {
                                cert_der,
                            })) => {
                                configuration = Some(Configuration {
                                    grpc_key_pem: key_pair.as_ref().map(|kp| kp.serialize_pem()),
                                    grpc_cert_pem: Some(
                                        defguard_certs::der_to_pem(
                                            &cert_der,
                                            &defguard_certs::PemLabel::Certificate,
                                        )
                                        .unwrap_or_default(),
                                    ),
                                });

                                if let Err(err) = tx.send(Ok(ProxySetupRequest {
                                    payload: Some(proxy_setup_request::Payload::Done(Done {})),
                                })) {
                                    error!("Failed to send Done message: {err}");
                                    return;
                                }

                                info!("Sent Done message to Defguard Core");
                            }
                            Some(proxy_setup_response::Payload::Done(Done {})) => {
                                info!("Received Done message from Defguard Core");
                                let lock = SETUP_CHANNEL.0.lock().await;
                                if let Some(configuration) = configuration.take() {
                                    if let Err(err) = lock.send(Some(configuration)).await {
                                        error!("Failed to send setup configuration: {err:?}");
                                    }
                                } else {
                                    error!(
                                        "No configuration available to send on setup completion"
                                    );
                                }

                                info!("Setup handshake completed successfully");
                            }
                            _ => {
                                error!(
                                    "Unexpected payload type while waiting for CsrAck: {:?}",
                                    response.payload
                                );
                                return;
                            }
                        },
                        Ok(None) => {
                            error!("gRPC stream has been closed unexpectedly");
                            return;
                        }
                        Err(err) => {
                            error!("gRPC client error while waiting for CertResponse: {err}");
                            return;
                        }
                    }
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
                if let Err(err) = GRPC_SERVER_RESTART_CHANNEL.0.lock().await.send(()).await {
                    error!("Failed to notify gRPC server restart: {err}");
                }
            }
            .instrument(tracing::Span::current()),
        );

        Ok(Response::new(UnboundedReceiverStream::new(rx)))
    }
}
