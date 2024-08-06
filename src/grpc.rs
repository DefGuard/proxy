use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc, Mutex,
    },
};

use tokio::sync::{mpsc, oneshot};
use tokio_stream::{wrappers::UnboundedReceiverStream, StreamExt};
use tonic::{Request, Response, Status, Streaming};

use crate::{
    error::ApiError,
    proto::{core_request, core_response, proxy_server, CoreRequest, CoreResponse, DeviceInfo},
};

// connected clients
type ClientMap = HashMap<SocketAddr, mpsc::UnboundedSender<Result<CoreRequest, Status>>>;

#[derive(Debug)]
pub(crate) struct ProxyServer {
    current_id: Arc<AtomicU64>,
    clients: Arc<Mutex<ClientMap>>,
    results: Arc<Mutex<HashMap<u64, oneshot::Sender<core_response::Payload>>>>,
    pub(crate) connected: Arc<AtomicBool>,
}

impl ProxyServer {
    #[must_use]
    /// Create new `ProxyServer`.
    pub fn new() -> Self {
        Self {
            current_id: Arc::new(AtomicU64::new(1)),
            clients: Arc::new(Mutex::new(HashMap::new())),
            results: Arc::new(Mutex::new(HashMap::new())),
            connected: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Sends message to the other side of RPC, with given `payload` and optional `device_info`.
    /// Returns `tokio::sync::oneshot::Reveicer` to let the caller await reply.
    #[instrument(name = "send_grpc_message", level = "debug", skip(self))]
    pub fn send(
        &self,
        payload: Option<core_request::Payload>,
        device_info: Option<DeviceInfo>,
    ) -> Result<oneshot::Receiver<core_response::Payload>, ApiError> {
        if let Some(client_tx) = self.clients.lock().unwrap().values().next() {
            let id = self.current_id.fetch_add(1, Ordering::Relaxed);
            let res = CoreRequest {
                id,
                device_info,
                payload,
            };
            if let Err(err) = client_tx.send(Ok(res)) {
                error!("Failed to send CoreRequest: {err}");
                return Err(ApiError::Unexpected("Failed to send CoreRequest".into()));
            };
            let (tx, rx) = oneshot::channel();
            let mut results = self.results.lock().unwrap();
            results.insert(id, tx);
            self.connected.store(true, Ordering::Relaxed);
            Ok(rx)
        } else {
            error!("Defguard core is disconnected");
            self.connected.store(false, Ordering::Relaxed);
            Err(ApiError::Unexpected("Defguard core is disconnected".into()))
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
        }
    }
}

#[tonic::async_trait]
impl proxy_server::Proxy for ProxyServer {
    type BidiStream = UnboundedReceiverStream<Result<CoreRequest, Status>>;

    /// Handle bidirectional communication with Defguard core.
    #[instrument(name = "bidirectional_communication", level = "debug", skip(self))]
    async fn bidi(
        &self,
        request: Request<Streaming<CoreResponse>>,
    ) -> Result<Response<Self::BidiStream>, Status> {
        let Some(address) = request.remote_addr() else {
            error!("Failed to determine client address for request: {request:?}");
            return Err(Status::internal("Failed to determine client address"));
        };
        info!("Defguard core RPC client connected from: {address}");

        let (tx, rx) = mpsc::unbounded_channel();
        self.clients.lock().unwrap().insert(address, tx);
        self.connected.store(true, Ordering::Relaxed);

        let clients = Arc::clone(&self.clients);
        let results = Arc::clone(&self.results);
        let connected = Arc::clone(&self.connected);
        let mut in_stream = request.into_inner();
        tokio::spawn(async move {
            while let Some(result) = in_stream.next().await {
                match result {
                    Ok(response) => {
                        debug!("Received message from Defguard core: {response:?}");
                        connected.store(true, Ordering::Relaxed);
                        // Discard empty payloads.
                        if let Some(payload) = response.payload {
                            if let Some(rx) = results.lock().unwrap().remove(&response.id) {
                                if let Err(err) = rx.send(payload) {
                                    error!("Failed to send message to rx: {err:?}");
                                }
                            } else {
                                error!("Missing receiver for response #{}", response.id);
                            }
                        }
                    }
                    Err(err) => error!("RPC client error: {err}"),
                }
            }
            info!("Defguard core client disconnected: {address}");
            connected.store(false, Ordering::Relaxed);
            clients.lock().unwrap().remove(&address);
        });

        Ok(Response::new(UnboundedReceiverStream::new(rx)))
    }
}
