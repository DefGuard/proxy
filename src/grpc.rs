use std::{
    any::Any,
    collections::HashMap,
    future::Future,
    net::SocketAddr,
    path::PathBuf,
    sync::{
        Arc, Mutex, RwLock,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
};

use axum_extra::extract::cookie::Key;
use defguard_certs::{CertificateError, CertificateInfo};
use defguard_grpc_tls::{certs::server_tls_config, server::certificate_serial_interceptor};
use defguard_version::{
    ComponentInfo, DefguardComponent, Version, get_tracing_variables,
    server::{DefguardVersionLayer, grpc::DefguardVersionInterceptor},
};
use tokio::{
    fs::remove_file,
    sync::{broadcast, mpsc, oneshot},
};
use tokio_stream::wrappers::UnboundedReceiverStream;
use tonic::{Request, Response, Status, Streaming, service::InterceptorLayer, transport::Server};
use tower::ServiceBuilder;
use tracing::Instrument;

use crate::{
    LogsReceiver, MIN_CORE_VERSION, VERSION, acme,
    acme::Port80Permit,
    error::ApiError,
    http::{CORE_CLIENT_CERT_NAME, GRPC_CA_CERT_NAME, GRPC_CERT_NAME, GRPC_KEY_NAME},
    proto::{
        AcmeCertificate, AcmeChallenge, AcmeIssueEvent, AcmeLogs, AcmeProgress, AcmeStep,
        CoreRequest, CoreResponse, DeviceInfo, acme_issue_event, core_request, core_response,
        proxy_server,
    },
};

// connected clients
type ClientMap = HashMap<SocketAddr, mpsc::UnboundedSender<Result<CoreRequest, Status>>>;

#[derive(Debug, Clone, Default)]
pub struct TlsConfig {
    pub grpc_key_pem: String,
    pub grpc_cert_pem: String,
    /// PEM-encoded CA certificate used to verify Core's mTLS client certificate chain.
    pub grpc_ca_cert_pem: String,
    /// DER-encoded Core client certificate; used to extract and pin the expected serial.
    pub core_client_cert_der: Vec<u8>,
}

pub(crate) struct ProxyServer {
    current_id: Arc<AtomicU64>,
    clients: Arc<RwLock<ClientMap>>,
    results: Arc<RwLock<HashMap<u64, oneshot::Sender<core_response::Payload>>>>,
    pub(crate) connected: Arc<AtomicBool>,
    pub(crate) core_version: Arc<Mutex<Option<Version>>>,
    tls_config: Arc<Mutex<Option<TlsConfig>>>,
    cookie_key: Arc<RwLock<Option<Key>>>,
    cert_dir: PathBuf,
    reset_tx: broadcast::Sender<()>,
    https_cert_tx: broadcast::Sender<(String, String)>,
    clear_https_tx: broadcast::Sender<()>,
    /// `Some` only when the main HTTP server is bound to port 80.
    /// Used to hand off port 80 gracefully during ACME HTTP-01 challenges.
    port80_pause_tx: Option<mpsc::Sender<(oneshot::Sender<()>, oneshot::Receiver<()>)>>,
    /// Shared log receiver - written by `GrpcLogLayer` for every tracing event.
    /// Drained during ACME execution to collect proxy log lines for error reporting.
    logs_rx: LogsReceiver,
    acme_staging: bool,
}

impl ProxyServer {
    /// Create new `ProxyServer`.
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        cookie_key: Arc<RwLock<Option<Key>>>,
        cert_dir: PathBuf,
        reset_tx: broadcast::Sender<()>,
        https_cert_tx: broadcast::Sender<(String, String)>,
        clear_https_tx: broadcast::Sender<()>,
        port80_pause_tx: Option<mpsc::Sender<(oneshot::Sender<()>, oneshot::Receiver<()>)>>,
        logs_rx: LogsReceiver,
        acme_staging: bool,
    ) -> Self {
        Self {
            cookie_key,
            current_id: Arc::new(AtomicU64::new(1)),
            clients: Arc::new(RwLock::new(HashMap::new())),
            results: Arc::new(RwLock::new(HashMap::new())),
            connected: Arc::new(AtomicBool::new(false)),
            core_version: Arc::new(Mutex::new(None)),
            tls_config: Arc::new(Mutex::new(None)),
            cert_dir,
            reset_tx,
            https_cert_tx,
            clear_https_tx,
            port80_pause_tx,
            logs_rx,
            acme_staging,
        }
    }

    pub(crate) fn configure(&self, config: TlsConfig) {
        let mut lock = self
            .tls_config
            .lock()
            .expect("Failed to acquire lock on config mutex when applying proxy configuration");
        *lock = Some(config);
    }

    pub(crate) fn get_tls_config(&self) -> Option<TlsConfig> {
        let lock = self
            .tls_config
            .lock()
            .expect("Failed to acquire lock on config mutex when retrieving proxy configuration");
        lock.clone()
    }

    pub(crate) async fn run<F>(self, addr: SocketAddr, shutdown: F) -> Result<(), anyhow::Error>
    where
        F: Future<Output = ()> + Send + 'static,
    {
        info!("Starting gRPC server on {addr}");
        let tls_config = self
            .get_tls_config()
            .ok_or_else(|| anyhow::anyhow!("gRPC server TLS configuration is missing"))?;

        // Extract Core client cert serial for pinning (None in no-TLS mode).
        let expected_serial = CertificateInfo::from_der(&tls_config.core_client_cert_der)
            .map_err(|e: CertificateError| anyhow::anyhow!("invalid core client cert DER: {e}"))?
            .serial;

        let tls_config = server_tls_config(
            &tls_config.grpc_cert_pem,
            &tls_config.grpc_key_pem,
            &tls_config.grpc_ca_cert_pem,
        )?;
        let mut builder = Server::builder().tls_config(tls_config)?;

        let own_version = Version::parse(VERSION)?;
        let versioned_service = ServiceBuilder::new()
            .layer(InterceptorLayer::new(certificate_serial_interceptor(
                expected_serial,
            )))
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
            .tls_config
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
            tls_config: Arc::clone(&self.tls_config),
            cert_dir: self.cert_dir.clone(),
            reset_tx: self.reset_tx.clone(),
            https_cert_tx: self.https_cert_tx.clone(),
            clear_https_tx: self.clear_https_tx.clone(),
            port80_pause_tx: self.port80_pause_tx.clone(),
            logs_rx: Arc::clone(&self.logs_rx),
            acme_staging: self.acme_staging,
        }
    }
}

#[tonic::async_trait]
impl proxy_server::Proxy for ProxyServer {
    type BidiStream = UnboundedReceiverStream<Result<CoreRequest, Status>>;
    type TriggerAcmeStream = UnboundedReceiverStream<Result<AcmeIssueEvent, Status>>;

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
        let https_cert_tx = self.https_cert_tx.clone();
        let clear_https_tx = self.clear_https_tx.clone();
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
                                    core_response::Payload::InitialInfo(info) => {
                                        info!("Received private cookies key");
                                        let key = Key::from(&info.private_cookies_key);
                                        *cookie_key.write().unwrap() = Some(key);
                                    }
                                    core_response::Payload::HttpsCerts(certs) => {
                                        info!("Received HTTPS certificates from Core");
                                        if let Err(err) =
                                            https_cert_tx.send((certs.cert_pem, certs.key_pem))
                                        {
                                            error!(
                                                "Failed to broadcast HTTPS certificates: {err}"
                                            );
                                        }
                                    }
                                    core_response::Payload::ClearHttpsCerts(_) => {
                                        info!("Received ClearHttpsCerts from Core");
                                        if let Err(err) = clear_https_tx.send(()) {
                                            error!("Failed to broadcast ClearHttpsCerts: {err}");
                                        }
                                    }
                                    other => {
                                        let maybe_rx = results.write().expect("Failed to acquire lock on results hashmap when processing response").remove(&response.id);
                                        if let Some(rx) = maybe_rx {
                                            if let Err(err) = rx.send(other) {
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
                clients.write().expect("Failed to acquire lock on clients hashmap when removing \
                    disconnected client").remove(&address);
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
        let ca_cert_path = self.cert_dir.join(GRPC_CA_CERT_NAME);
        let core_client_cert_path = self.cert_dir.join(CORE_CLIENT_CERT_NAME);

        let remove_cert_file = async |path: &std::path::Path, label: &str| -> Result<(), Status> {
            if let Err(err) = remove_file(path).await
                && err.kind() != std::io::ErrorKind::NotFound
            {
                error!("Failed to remove {label} at {}: {err}", path.display());
                return Err(Status::internal(format!("Failed to remove {label}")));
            }
            info!("Removed {label} at {}", path.display());
            Ok(())
        };

        remove_cert_file(&cert_path, "gRPC certificate").await?;
        remove_cert_file(&key_path, "gRPC key").await?;
        remove_cert_file(&ca_cert_path, "CA certificate").await?;
        remove_cert_file(&core_client_cert_path, "Core client certificate").await?;

        *self
            .tls_config
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

    #[instrument(skip(self, request))]
    async fn trigger_acme(
        &self,
        request: Request<AcmeChallenge>,
    ) -> Result<Response<Self::TriggerAcmeStream>, Status> {
        let challenge = request.into_inner();
        let domain = challenge.domain.clone();
        let existing_credentials = challenge.account_credentials_json.clone();

        info!("Starting ACME HTTP-01 for domain: {domain}");

        let (tx, rx) = mpsc::unbounded_channel::<Result<AcmeIssueEvent, Status>>();

        // Drain any stale log entries accumulated before this ACME run started.
        {
            let mut rx_guard = self.logs_rx.lock().await;
            while rx_guard.try_recv().is_ok() {}
        }

        let pause_tx = self.port80_pause_tx.clone();
        let logs_rx = Arc::clone(&self.logs_rx);
        let acme_staging = self.acme_staging;
        tokio::spawn(async move {
            // Request a graceful hand-off of port 80 from the main HTTP server if it is bound
            // there, so the ACME challenge listener can bind.
            let permit: Option<Port80Permit> = if let Some(ref pause_tx) = pause_tx {
                let (ready_tx, ready_rx) = oneshot::channel::<()>();
                let (done_tx, done_rx) = oneshot::channel::<()>();
                if pause_tx.send((ready_tx, done_rx)).await.is_err() {
                    error!(
                        "Failed to request port-80 hand-off for ACME; \
                         HTTP server may have stopped"
                    );
                    let _ = tx.send(Err(Status::internal(
                        "Failed to request port-80 hand-off for ACME",
                    )));
                    return;
                }
                Some(Port80Permit {
                    ready: ready_rx,
                    done_tx,
                })
            } else {
                None
            };

            // Channel used by run_acme_http01 to emit intermediate progress steps.
            let (progress_tx, mut progress_rx) = mpsc::unbounded_channel::<AcmeStep>();

            // Forward progress steps from acme.rs onto the gRPC response stream.
            let tx_fwd = tx.clone();
            tokio::spawn(async move {
                while let Some(step) = progress_rx.recv().await {
                    let event = AcmeIssueEvent {
                        payload: Some(acme_issue_event::Payload::Progress(AcmeProgress {
                            step: step as i32,
                        })),
                    };
                    if tx_fwd.send(Ok(event)).is_err() {
                        // Core disconnected - stop forwarding.
                        break;
                    }
                }
            });

            match acme::run_acme_http01(
                domain.clone(),
                existing_credentials,
                acme_staging,
                permit,
                progress_tx,
            )
            .await
            {
                Ok(acme_result) => {
                    let cert_event = AcmeIssueEvent {
                        payload: Some(acme_issue_event::Payload::Certificate(AcmeCertificate {
                            cert_pem: acme_result.cert_pem,
                            key_pem: acme_result.key_pem,
                            account_credentials_json: acme_result.account_credentials_json,
                        })),
                    };
                    if tx.send(Ok(cert_event)).is_err() {
                        error!(
                            "ACME result stream receiver disconnected before cert could be sent"
                        );
                    } else {
                        info!("ACME certificate for domain '{domain}' streamed to Core");
                    }
                }
                Err(err) => {
                    let chain = err
                        .chain()
                        .map(std::string::ToString::to_string)
                        .collect::<Vec<_>>()
                        .join(": ");
                    error!("ACME HTTP-01 failed for domain '{domain}': {chain}");

                    // Drain collected log lines and forward them to Core before the error status.
                    let log_lines: Vec<String> = {
                        let mut rx_guard = logs_rx.lock().await;
                        let mut lines = Vec::new();
                        while let Ok(entry) = rx_guard.try_recv() {
                            lines.push(format!(
                                "{} {} {}: message={}",
                                entry.timestamp,
                                entry.level.to_uppercase(),
                                entry.target,
                                entry.message
                            ));
                        }
                        lines
                    };
                    if !log_lines.is_empty() {
                        let _ = tx.send(Ok(AcmeIssueEvent {
                            payload: Some(acme_issue_event::Payload::Logs(AcmeLogs {
                                lines: log_lines,
                            })),
                        }));
                    }

                    let _ = tx.send(Err(Status::internal(format!(
                        "ACME HTTP-01 certificate issuance failed: {chain}"
                    ))));
                }
            }
        });

        Ok(Response::new(UnboundedReceiverStream::new(rx)))
    }
}
