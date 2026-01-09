use std::{
    any::Any,
    collections::HashMap,
    net::SocketAddr,
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc, Mutex,
    },
};

use defguard_version::{get_tracing_variables, ComponentInfo, DefguardComponent, Version};
use tokio::sync::{mpsc, oneshot};
use tokio_stream::wrappers::UnboundedReceiverStream;
use tonic::{Request, Response, Status, Streaming};
use tracing::Instrument;

use crate::{
    error::ApiError,
    proto::{core_request, core_response, proxy_server, CoreRequest, CoreResponse, DeviceInfo},
};

// connected clients
type ClientMap = HashMap<SocketAddr, mpsc::UnboundedSender<Result<CoreRequest, Status>>>;
static COOKIE_KEY_HEADER: &str = "dg-cookie-key-bin";

pub(crate) struct ProxyServer {
    current_id: Arc<AtomicU64>,
    clients: Arc<Mutex<ClientMap>>,
    results: Arc<Mutex<HashMap<u64, oneshot::Sender<core_response::Payload>>>>,
    http_channel: mpsc::UnboundedSender<Vec<u8>>,
    pub(crate) connected: Arc<AtomicBool>,
    pub(crate) core_version: Arc<Mutex<Option<Version>>>,
}

impl ProxyServer {
    #[must_use]
    /// Create new `ProxyServer`.
    pub(crate) fn new(http_channel: mpsc::UnboundedSender<Vec<u8>>) -> Self {
        Self {
            http_channel,
            current_id: Arc::new(AtomicU64::new(1)),
            clients: Arc::new(Mutex::new(HashMap::new())),
            results: Arc::new(Mutex::new(HashMap::new())),
            connected: Arc::new(AtomicBool::new(false)),
            core_version: Arc::new(Mutex::new(None)),
        }
    }

    /// Sends message to the other side of RPC, with given `payload` and optional `device_info`.
    /// Returns `tokio::sync::oneshot::Reveicer` to let the caller await reply.
    #[instrument(name = "send_grpc_message", level = "debug", skip(self, payload))]
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
            let mut results = self.results.lock().unwrap();
            results.insert(id, tx);
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
}

impl Clone for ProxyServer {
    fn clone(&self) -> Self {
        Self {
            current_id: Arc::clone(&self.current_id),
            clients: Arc::clone(&self.clients),
            results: Arc::clone(&self.results),
            connected: Arc::clone(&self.connected),
            core_version: Arc::clone(&self.core_version),
            http_channel: self.http_channel.clone(),
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
        let cookie_key = request.metadata().get_bin(COOKIE_KEY_HEADER).unwrap();
        let key = (cookie_key.to_bytes().unwrap())
            .into_iter()
			.collect::<Vec<_>>();
        let _ = self.http_channel.send(key);
        let Some(address) = request.remote_addr() else {
            error!("Failed to determine client address for request: {request:?}");
            return Err(Status::internal("Failed to determine client address"));
        };
        let maybe_info = ComponentInfo::from_metadata(request.metadata());
        let (version, info) = get_tracing_variables(&maybe_info);
        let mut core_version = self.core_version.lock().unwrap();
        *core_version = Some(version.clone());

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
                            // Discard empty payloads.
                            if let Some(payload) = response.payload {
                                if let Some(rx) = results.lock().unwrap().remove(&response.id) {
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
