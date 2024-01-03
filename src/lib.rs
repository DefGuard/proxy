pub mod config;
pub mod error;
mod grpc;
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
use tracing::{debug, info};

use proto::{proxy_request, proxy_response, proxy_server, ProxyRequest, ProxyResponse};

// connected clients
type ClientMap = HashMap<SocketAddr, mpsc::UnboundedSender<Result<ProxyResponse, Status>>>;

#[derive(Debug)]
pub(crate) struct ProxyServer {
    current_id: Arc<AtomicU64>,
    clients: Arc<Mutex<ClientMap>>,
    results: Arc<Mutex<HashMap<u64, oneshot::Sender<proxy_request::Payload>>>>,
}

impl ProxyServer {
    #[must_use]
    pub fn new() -> Self {
        Self {
            current_id: Arc::new(AtomicU64::new(1)),
            clients: Arc::new(Mutex::new(HashMap::new())),
            results: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    #[must_use]
    pub fn send(
        &self,
        payload: Option<proxy_response::Payload>,
    ) -> Option<oneshot::Receiver<proxy_request::Payload>> {
        if let Some(client_tx) = self.clients.lock().unwrap().values().next() {
            let id = self.current_id.fetch_add(1, Ordering::Relaxed);
            let res = ProxyResponse { id, payload };
            if client_tx.send(Ok(res)).is_ok() {
                let (tx, rx) = oneshot::channel();
                self.results.lock().unwrap().insert(id, tx);
                return Some(rx);
            } else {
                debug!("Failed to send ProxyResponse");
            }
        }

        None
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
    type BidiStream = UnboundedReceiverStream<Result<ProxyResponse, Status>>;

    async fn bidi(
        &self,
        request: Request<Streaming<ProxyRequest>>,
    ) -> Result<Response<Self::BidiStream>, Status> {
        let Some(address) = request.remote_addr() else {
            return Err(Status::internal("failed to determine client address"));
        };
        info!("gRPC client connected from: {address}");

        let (tx, rx) = mpsc::unbounded_channel();
        self.clients.lock().unwrap().insert(address, tx);

        let clients = Arc::clone(&self.clients);
        let results = Arc::clone(&self.results);
        let mut in_stream = request.into_inner();
        tokio::spawn(async move {
            while let Some(result) = in_stream.next().await {
                match result {
                    Ok(response) => {
                        debug!("bidi received {response:?}");
                        // Discard empty payloads.
                        if let Some(payload) = response.payload {
                            if let Some(rx) = results.lock().unwrap().remove(&response.id) {
                                if rx.send(payload).is_err() {
                                    debug!("Failed to send to rx");
                                }
                            } else {
                                debug!("No oneshot receiver for {}", response.id);
                            }
                        }
                    }
                    Err(err) => info!("bidi client error: {err}"),
                }
            }
            debug!("Client disconnected {address}");
            if let Ok(mut clients) = clients.lock() {
                clients.remove(&address);
            }
        });

        Ok(Response::new(UnboundedReceiverStream::new(rx)))
    }
}
