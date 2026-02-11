use std::{
    any::Any,
    collections::HashMap,
    future::Future,
    net::SocketAddr,
    path::PathBuf,
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
use tokio::sync::{broadcast, mpsc, oneshot};
use tokio_stream::wrappers::UnboundedReceiverStream;
use tonic::{
    transport::{Identity, Server, ServerTlsConfig},
    Request, Response, Status, Streaming,
};
use tower::ServiceBuilder;
use tracing::Instrument;

use crate::{
    error::ApiError,
    http::{GRPC_CERT_NAME, GRPC_KEY_NAME},
    proto::{core_request, core_response, proxy_server, CoreRequest, CoreResponse, DeviceInfo},
    MIN_CORE_VERSION, VERSION,
};

// connected clients
type ClientMap = HashMap<SocketAddr, mpsc::UnboundedSender<Result<CoreRequest, Status>>>;

#[derive(Debug, Clone, Default)]
pub struct Configuration {
    pub grpc_key_pem: String,
    pub grpc_cert_pem: String,
}

pub(crate) struct ProxyServer {
    current_id: Arc<AtomicU64>,
    clients: Arc<RwLock<ClientMap>>,
    results: Arc<RwLock<HashMap<u64, oneshot::Sender<core_response::Payload>>>>,
    pub(crate) connected: Arc<AtomicBool>,
    pub(crate) core_version: Arc<Mutex<Option<Version>>>,
    config: Arc<Mutex<Option<Configuration>>>,
    cookie_key: Arc<RwLock<Option<Key>>>,
    cert_dir: PathBuf,
    reset_tx: broadcast::Sender<()>,
}

impl ProxyServer {
    #[must_use]
    /// Create new `ProxyServer`.
    pub(crate) fn new(
        cookie_key: Arc<RwLock<Option<Key>>>,
        cert_dir: PathBuf,
        reset_tx: broadcast::Sender<()>,
    ) -> Self {
        Self {
            cookie_key,
            current_id: Arc::new(AtomicU64::new(1)),
            clients: Arc::new(RwLock::new(HashMap::new())),
            results: Arc::new(RwLock::new(HashMap::new())),
            connected: Arc::new(AtomicBool::new(false)),
            core_version: Arc::new(Mutex::new(None)),
            config: Arc::new(Mutex::new(None)),
            cert_dir,
            reset_tx,
        }
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

    pub(crate) async fn run<F>(
        self,
        addr: SocketAddr,
        shutdown: F,
    ) -> Result<(), anyhow::Error>
    where
        F: Future<Output = ()> + Send + 'static,
    {
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
            .serve_with_shutdown(addr, shutdown)
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
            .read()
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
                .write()
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
            cert_dir: self.cert_dir.clone(),
            reset_tx: self.reset_tx.clone(),
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
            .write()
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
                                        let maybe_rx = results.write().expect("Failed to acquire lock on results hashmap when processing response").remove(&response.id);
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
                clients.write().expect("Failed to acquire lock on clients hashmap when removing disconnected client").remove(&address);
            }
            .instrument(tracing::Span::current()),
        );

        Ok(Response::new(UnboundedReceiverStream::new(rx)))
    }

    #[instrument(skip(self, _request))]
    async fn purge(&self, _request: Request<()>) -> Result<Response<()>, Status> {
        debug!("Received purge request, removing gRPC certificate files");
        let cert_path = self.cert_dir.join(GRPC_CERT_NAME);
        let key_path = self.cert_dir.join(GRPC_KEY_NAME);

        if let Err(err) = tokio::fs::remove_file(&cert_path).await {
            if err.kind() != std::io::ErrorKind::NotFound {
                error!(
                    "Failed to remove gRPC certificate at {:?}: {err}",
                    cert_path
                );
                return Err(Status::internal("Failed to remove gRPC certificate"));
            }
        }

        if let Err(err) = tokio::fs::remove_file(&key_path).await {
            if err.kind() != std::io::ErrorKind::NotFound {
                error!("Failed to remove gRPC key at {:?}: {err}", key_path);
                return Err(Status::internal("Failed to remove gRPC key"));
            }
        }

        *self
            .config
            .lock()
            .expect("Failed to lock config mutex during purge") = None;
        *self
            .core_version
            .lock()
            .expect("Failed to lock core_version mutex during purge") = None;
        *self
            .cookie_key
            .write()
            .expect("Failed to lock cookie key during purge") = None;
        self.connected.store(false, Ordering::Relaxed);

        if self.reset_tx.send(()).is_err() {
            error!("Failed to notify reset handler");
            return Err(Status::internal("Failed to restart setup process"));
        }

        info!("Removed gRPC certificate files; entering setup mode");
        Ok(Response::new(()))
    }
}
