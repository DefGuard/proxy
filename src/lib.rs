pub mod config;
pub mod error;
mod handlers;
pub mod server;

pub(crate) mod proto {
    tonic::include_proto!("defguard.proxy");
}

use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Mutex,
    },
};

use tokio::sync::{mpsc, oneshot};
use tokio_stream::{wrappers::UnboundedReceiverStream, StreamExt};
use tonic::{Request, Response, Status, Streaming};

use crate::error::ApiError;
use proto::{core_request, core_response, proxy_server, CoreRequest, CoreResponse, DeviceInfo};

#[macro_use]
extern crate tracing;

// connected clients
type ClientMap = HashMap<SocketAddr, mpsc::UnboundedSender<Result<CoreRequest, Status>>>;

#[derive(Debug)]
pub(crate) struct ProxyServer {
    current_id: Arc<AtomicU64>,
    clients: Arc<Mutex<ClientMap>>,
    results: Arc<Mutex<HashMap<u64, oneshot::Sender<core_response::Payload>>>>,
}

impl ProxyServer {
    #[must_use]
    /// Create new `ProxyServer`.
    pub fn new() -> Self {
        Self {
            current_id: Arc::new(AtomicU64::new(1)),
            clients: Arc::new(Mutex::new(HashMap::new())),
            results: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Sends message to the other side of RPC, with given `payload` and optional 'device_info`.
    /// Returns `tokio::sync::oneshot::Reveicer` to let the caller await reply.
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
            if client_tx.send(Ok(res)).is_ok() {
                let (tx, rx) = oneshot::channel();
                if let Ok(mut results) = self.results.lock() {
                    results.insert(id, tx);
                    return Ok(rx);
                }
            }

            debug!("Failed to send CoreRequest");
        }

        Err(ApiError::Unexpected(
            "Failed to communicate with Defguard core".into(),
        ))
    }
}

impl Clone for ProxyServer {
    fn clone(&self) -> Self {
        Self {
            current_id: Arc::clone(&self.current_id),
            clients: Arc::clone(&self.clients),
            results: Arc::clone(&self.results),
        }
    }
}

#[tonic::async_trait]
impl proxy_server::Proxy for ProxyServer {
    type BidiStream = UnboundedReceiverStream<Result<CoreRequest, Status>>;

    /// Handle bidirectional communication with Defguard core.
    async fn bidi(
        &self,
        request: Request<Streaming<CoreResponse>>,
    ) -> Result<Response<Self::BidiStream>, Status> {
        let Some(address) = request.remote_addr() else {
            return Err(Status::internal("failed to determine client address"));
        };
        info!("RPC client connected from: {address}");

        let (tx, rx) = mpsc::unbounded_channel();
        self.clients.lock().unwrap().insert(address, tx);

        let clients = Arc::clone(&self.clients);
        let results = Arc::clone(&self.results);
        let mut in_stream = request.into_inner();
        tokio::spawn(async move {
            while let Some(result) = in_stream.next().await {
                match result {
                    Ok(response) => {
                        debug!("RPC message received {response:?}");
                        // Discard empty payloads.
                        if let Some(payload) = response.payload {
                            if let Ok(mut results) = results.lock() {
                                if let Some(rx) = results.remove(&response.id) {
                                    if rx.send(payload).is_err() {
                                        debug!("failed to send to rx");
                                    }
                                } else {
                                    debug!("missing receiver for response #{}", response.id);
                                }
                            } else {
                                error!("failed to obtain mutex on results");
                            }
                        }
                    }
                    Err(err) => error!("RPC client error: {err}"),
                }
            }
            debug!("client disconnected {address}");
            if let Ok(mut clients) = clients.lock() {
                clients.remove(&address);
            } else {
                error!("failed to obtain mutex on clients");
            }
        });

        Ok(Response::new(UnboundedReceiverStream::new(rx)))
    }
}
