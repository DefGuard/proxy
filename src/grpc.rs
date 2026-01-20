use std::{
    any::Any,
    collections::HashMap,
    net::SocketAddr,
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc, Mutex, RwLock,
    },
};

use axum_extra::extract::cookie::Key;
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
    proto::{core_request, core_response, proxy_server, CoreRequest, CoreResponse, DeviceInfo},
    MIN_CORE_VERSION, VERSION,
};

// connected clients
type ClientMap = HashMap<SocketAddr, mpsc::UnboundedSender<Result<CoreRequest, Status>>>;

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
    cookie_key: Arc<RwLock<Option<Key>>>,
    setup_in_progress: Arc<AtomicBool>,
}

impl ProxyServer {
    #[must_use]
    /// Create new `ProxyServer`.
    pub(crate) fn new(cookie_key: Arc<RwLock<Option<Key>>>) -> Self {
        Self {
            cookie_key,
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
        let mut lock = self
            .config
            .lock()
            .expect("Failed to acquire lock on config mutex when updating TLS configuration");
        let config = lock.get_or_insert_with(Configuration::default);
        config.grpc_cert_pem = cert_pem;
        config.grpc_key_pem = key_pem;
        Ok(())
    }

    pub(crate) fn configure(&self, config: Configuration) {
        let mut lock = self
            .config
            .lock()
            .expect("Failed to acquire lock on config mutex when applying proxy configuration");
        *lock = Some(config);
    }

    pub(crate) fn get_configuration(&self) -> Option<Configuration> {
        let lock = self
            .config
            .lock()
            .expect("Failed to acquire lock on config mutex when retrieving proxy configuration");
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
        if let Some(client_tx) = self
            .clients
            .lock()
            .expect("Failed to acquire lock on clients hashmap when sending message to core")
            .values()
            .next()
        {
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
            self.results
                .lock()
                .expect("Failed to acquire lock on results hashmap when sending CoreRequest")
                .insert(id, tx);
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
        let lock = self
            .config
            .lock()
            .expect("Failed to acquire lock on config mutex when checking setup status");
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
            cookie_key: Arc::clone(&self.cookie_key),
            config: Arc::clone(&self.config),
            setup_in_progress: Arc::clone(&self.setup_in_progress),
        }
    }
}

#[tonic::async_trait]
impl proxy_server::Proxy for ProxyServer {
    type BidiStream = UnboundedReceiverStream<Result<CoreRequest, Status>>;

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
        *self.core_version.lock().expect(
            "Failed to acquire lock on core_version mutex when storing version information",
        ) = Some(version.clone());

        let span = tracing::info_span!("core_bidi_stream", component = %DefguardComponent::Core,
            version = version.to_string(), info);
        let _guard = span.enter();

        info!("Defguard Core gRPC client connected from: {address}");
        let (tx, rx) = mpsc::unbounded_channel();
        self.clients
            .lock()
            .expect(
                "Failed to acquire lock on clients hashmap when registering new core connection",
            )
            .insert(address, tx);
        self.connected.store(true, Ordering::Relaxed);

        let clients = Arc::clone(&self.clients);
        let results = Arc::clone(&self.results);
        let connected = Arc::clone(&self.connected);
        let cookie_key = Arc::clone(&self.cookie_key);
        tokio::spawn(
            async move {
                let mut stream = request.into_inner();
                loop {
                    match stream.message().await {
                        Ok(Some(response)) => {
                            debug!("Received message from Defguard Core ID={}", response.id);
                            connected.store(true, Ordering::Relaxed);
                            if let Some(payload) = response.payload {
                                match payload {
                                    core_response::Payload::InitialInfo(payload) => {
                                        info!("Received private cookies key");
                                        let key = Key::from(&payload.private_cookies_key);
                                        *cookie_key.write().unwrap() = Some(key);
                                    },
                                    _ => {
                                        let maybe_rx = results.lock().expect("Failed to acquire lock on results hashmap when processing response").remove(&response.id);
                                        if let Some(rx) = maybe_rx {
                                            if let Err(err) = rx.send(payload) {
                                                error!("Failed to send message to rx {:?}", err.type_id());
                                            }
                                        } else {
                                            error!("Missing receiver for response #{}", response.id);
                                        }
                                    }
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
                clients.lock().expect("Failed to acquire lock on clients hashmap when removing disconnected client").remove(&address);
            }
            .instrument(tracing::Span::current()),
        );

        Ok(Response::new(UnboundedReceiverStream::new(rx)))
    }
}
