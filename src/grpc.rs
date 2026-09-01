use std::{
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
use url::Url;

use crate::{
    LogsReceiver, MIN_CORE_VERSION, VERSION, acme,
    acme::Port80Permit,
    error::ApiError,
    http::{CORE_CLIENT_CERT_NAME, GRPC_CA_CERT_NAME, GRPC_CERT_NAME, GRPC_KEY_NAME},
    proto::{
        AcmeCertificate, AcmeChallenge, AcmeIssueEvent, AcmeLogs, AcmeProgress, AcmeStep,
        CoreRequest, CoreResponse, DeviceInfo, PublicSettings, acme_issue_event, core_request,
        core_response, proxy_server,
    },
};

// connected clients
type ClientMap = HashMap<SocketAddr, mpsc::UnboundedSender<Result<CoreRequest, Status>>>;

// requests awaiting a `CoreResponse`, keyed by the id stamped into the outgoing `CoreRequest`
type ResultMap = HashMap<u64, oneshot::Sender<core_response::Payload>>;

/// Hand a `CoreResponse` payload to the request waiting on it.
///
/// [`ProxyServer::send`] registers a oneshot sender under the id it stamps into the outgoing
/// `CoreRequest`; Core echoes that id back and this is the lookup that pairs the two. Returns
/// whether the payload reached a waiting receiver: it does not when Core answers an id we never
/// sent, or answers twice, or when the caller gave up before the reply arrived.
fn resolve_response(results: &RwLock<ResultMap>, id: u64, payload: core_response::Payload) -> bool {
    let Some(tx) = results
        .write()
        .expect("Failed to acquire lock on results hashmap when resolving response")
        .remove(&id)
    else {
        error!("Missing receiver for response #{id}");
        return false;
    };
    if tx.send(payload).is_err() {
        error!("Receiver for response #{id} is gone; the request was likely canceled");
        return false;
    }
    true
}

#[derive(Clone)]
pub(crate) struct PublicSettingsState {
    pub(crate) display_password_reset: Arc<AtomicBool>,
    pub(crate) display_download_step: Arc<AtomicBool>,
    pub(crate) cookie_secure: Arc<AtomicBool>,
}

impl Default for PublicSettingsState {
    fn default() -> Self {
        Self {
            display_password_reset: Arc::new(AtomicBool::new(true)),
            display_download_step: Arc::new(AtomicBool::new(true)),
            cookie_secure: Arc::new(AtomicBool::new(true)),
        }
    }
}

impl PublicSettingsState {
    pub(crate) fn apply(&self, settings: PublicSettings) {
        self.display_password_reset
            .store(settings.display_password_reset, Ordering::Relaxed);
        self.display_download_step
            .store(settings.display_download_step, Ordering::Relaxed);

        // Default to `Secure` whenever the URL does not explicitly say the proxy is reached over
        // plaintext HTTP. A schemeless value like `proxy.example.com:8443` parses successfully with
        // the hostname as its scheme, so matching on `https` alone would silently drop the flag.
        let secure = match settings.public_url.as_deref() {
            None | Some("") => true,
            Some(public_url) => match Url::parse(public_url) {
                Ok(url) => match url.scheme() {
                    "https" => true,
                    "http" => false,
                    scheme => {
                        warn!(
                            "Unexpected scheme {scheme:?} in public proxy URL from Core \
                            ({public_url:?}); defaulting to secure cookies"
                        );
                        true
                    }
                },
                Err(err) => {
                    warn!("Failed to parse public proxy URL from Core ({public_url:?}): {err}");
                    true
                }
            },
        };
        self.cookie_secure.store(secure, Ordering::Relaxed);
    }
}

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
    results: Arc<RwLock<ResultMap>>,
    pub(crate) connected: Arc<AtomicBool>,
    pub(crate) core_version: Arc<Mutex<Option<Version>>>,
    /// Public settings received from Core and shared with HTTP handlers.
    pub(crate) public_settings: PublicSettingsState,
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
            public_settings: PublicSettingsState::default(),
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

#[cfg(test)]
impl ProxyServer {
    pub(crate) fn register_test_client(
        &self,
    ) -> mpsc::UnboundedReceiver<Result<CoreRequest, Status>> {
        let (tx, rx) = mpsc::unbounded_channel();
        self.clients
            .write()
            .expect("Failed to acquire lock on clients hashmap when registering test client")
            .insert(SocketAddr::from(([127, 0, 0, 1], 0)), tx);
        self.connected.store(true, Ordering::Relaxed);
        rx
    }

    /// Resolve a pending request through the same dispatch the gRPC response loop uses, so tests
    /// driving a full flow exercise that path rather than mirroring it.
    pub(crate) fn resolve_test_response(&self, id: u64, payload: core_response::Payload) {
        assert!(
            resolve_response(&self.results, id, payload),
            "Response #{id} did not reach a waiting receiver"
        );
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
            public_settings: self.public_settings.clone(),
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
        let public_settings = self.public_settings.clone();
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
                                            error!("Failed to broadcast HTTPS certificates: {err}");
                                        }
                                    }
                                    core_response::Payload::ClearHttpsCerts(_) => {
                                        info!("Received ClearHttpsCerts from Core");
                                        if let Err(err) = clear_https_tx.send(()) {
                                            error!("Failed to broadcast ClearHttpsCerts: {err}");
                                        }
                                    }
                                    core_response::Payload::PublicSettings(settings) => {
                                        debug!("Received PublicSettings from Core");
                                        public_settings.apply(settings);
                                    }
                                    other => {
                                        resolve_response(&results, response.id, other);
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
                clients
                    .write()
                    .expect(
                        "Failed to acquire lock on clients hashmap when removing \
                    disconnected client",
                    )
                    .remove(&address);
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
            match remove_file(path).await {
                Ok(()) => {
                    info!("Removed {label} at {}", path.display());
                    Ok(())
                }
                Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                    debug!("{label} not found at {}, skipping removal", path.display());
                    Ok(())
                }
                Err(err) => {
                    error!("Failed to remove {label} at {}: {err}", path.display());
                    Err(Status::internal(format!("Failed to remove {label}")))
                }
            }
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

#[cfg(test)]
mod tests {
    use std::sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    };

    use super::{core_response::Payload, *};
    use crate::{
        proto::{EnrollmentStartRequest, EnrollmentStartResponse},
        tests::support::{test_proxy_server, test_public_settings},
    };

    fn test_device_info() -> DeviceInfo {
        DeviceInfo {
            ip_address: "127.0.0.1".to_owned(),
            user_agent: None,
            version: None,
            platform: None,
        }
    }

    fn applied_public_settings(
        settings: PublicSettings,
        initial_cookie_secure: bool,
    ) -> PublicSettingsState {
        let state = PublicSettingsState {
            display_password_reset: Arc::new(AtomicBool::new(false)),
            display_download_step: Arc::new(AtomicBool::new(false)),
            cookie_secure: Arc::new(AtomicBool::new(initial_cookie_secure)),
        };
        state.apply(settings);
        state
    }

    fn pending_request(results: &RwLock<ResultMap>, id: u64) -> oneshot::Receiver<Payload> {
        let (tx, rx) = oneshot::channel();
        results
            .write()
            .expect("results lock poisoned")
            .insert(id, tx);
        rx
    }

    fn enrollment_response(deadline_timestamp: i64) -> Payload {
        Payload::EnrollmentStart(EnrollmentStartResponse {
            admin: None,
            user: None,
            deadline_timestamp,
            final_page_content: String::new(),
            instance: None,
            settings: None,
        })
    }

    /// `send` must stamp the same id into the outgoing `CoreRequest` and into the `results` key,
    /// or the response loop looks up an entry that was never registered.
    #[test]
    fn test_send_registers_the_id_it_puts_on_the_wire() {
        let server = test_proxy_server(Arc::default());
        let mut client = server.register_test_client();

        let _response_rx = server
            .send(
                core_request::Payload::EnrollmentStart(EnrollmentStartRequest {
                    token: "enrollment-token".to_owned(),
                }),
                test_device_info(),
            )
            .expect("Failed to send enrollment request");

        let request = client
            .try_recv()
            .expect("No CoreRequest reached the client channel")
            .expect("Test client received an error");
        assert!(matches!(
            request.payload,
            Some(core_request::Payload::EnrollmentStart(_))
        ));
        assert_eq!(request.device_info, Some(test_device_info()));
        assert!(
            server
                .results
                .read()
                .expect("results lock poisoned")
                .contains_key(&request.id),
            "Request #{} went out without a registered receiver",
            request.id
        );
        assert!(server.connected.load(Ordering::Relaxed));
    }

    #[test]
    fn test_send_fails_when_core_is_not_connected() {
        let server = test_proxy_server(Arc::default());
        server.connected.store(true, Ordering::Relaxed);

        let result = server.send(
            core_request::Payload::EnrollmentStart(EnrollmentStartRequest {
                token: "enrollment-token".to_owned(),
            }),
            test_device_info(),
        );

        assert!(result.is_err());
        assert!(!server.connected.load(Ordering::Relaxed));
    }

    #[test]
    fn test_resolve_response_delivers_to_the_waiting_request() {
        let results = RwLock::new(ResultMap::new());
        let mut rx = pending_request(&results, 7);

        assert!(resolve_response(&results, 7, enrollment_response(1)));

        assert!(matches!(
            rx.try_recv().expect("Response never arrived"),
            Payload::EnrollmentStart(response) if response.deadline_timestamp == 1
        ));
    }

    /// Each request must get its own reply. Resolving out of order is the ordinary case, since
    /// Core answers whenever each request completes rather than in the order they were sent.
    #[test]
    fn test_resolve_response_does_not_cross_talk_between_requests() {
        let results = RwLock::new(ResultMap::new());
        let mut first = pending_request(&results, 1);
        let mut second = pending_request(&results, 2);

        assert!(resolve_response(&results, 2, enrollment_response(22)));
        assert!(resolve_response(&results, 1, enrollment_response(11)));

        assert!(matches!(
            first.try_recv().expect("Response never arrived"),
            Payload::EnrollmentStart(response) if response.deadline_timestamp == 11
        ));
        assert!(matches!(
            second.try_recv().expect("Response never arrived"),
            Payload::EnrollmentStart(response) if response.deadline_timestamp == 22
        ));
    }

    /// An id Core makes up, or answers twice, must be dropped rather than panicking the response
    /// loop or leaving a stale entry behind.
    #[test]
    fn test_resolve_response_reports_an_unknown_id() {
        let results = RwLock::new(ResultMap::new());
        let mut rx = pending_request(&results, 7);

        assert!(!resolve_response(&results, 8, enrollment_response(1)));
        assert!(rx.try_recv().is_err());

        assert!(resolve_response(&results, 7, enrollment_response(1)));
        assert!(
            !resolve_response(&results, 7, enrollment_response(1)),
            "A resolved request must not stay in the map"
        );
    }

    #[test]
    fn test_resolve_response_reports_an_abandoned_request() {
        let results = RwLock::new(ResultMap::new());
        drop(pending_request(&results, 7));

        assert!(!resolve_response(&results, 7, enrollment_response(1)));
        assert!(
            results.read().expect("results lock poisoned").is_empty(),
            "An abandoned request must not stay in the map"
        );
    }

    #[test]
    fn test_public_settings_set_https_secure_and_update_display_flags() {
        let state = applied_public_settings(
            PublicSettings {
                display_password_reset: true,
                display_download_step: false,
                public_url: Some("https://edge.example.com".to_owned()),
            },
            false,
        );

        assert!(state.display_password_reset.load(Ordering::Relaxed));
        assert!(!state.display_download_step.load(Ordering::Relaxed));
        assert!(state.cookie_secure.load(Ordering::Relaxed));
    }

    #[test]
    fn test_public_settings_set_http_insecure() {
        let state = applied_public_settings(
            PublicSettings {
                display_password_reset: false,
                display_download_step: true,
                public_url: Some("http://edge.example.com".to_owned()),
            },
            true,
        );

        assert!(!state.display_password_reset.load(Ordering::Relaxed));
        assert!(state.display_download_step.load(Ordering::Relaxed));
        assert!(!state.cookie_secure.load(Ordering::Relaxed));
    }

    #[test]
    fn test_public_settings_default_to_secure_without_url() {
        for public_url in [None, Some("")] {
            let state = applied_public_settings(test_public_settings(public_url), false);

            assert!(state.display_password_reset.load(Ordering::Relaxed));
            assert!(state.display_download_step.load(Ordering::Relaxed));
            assert!(state.cookie_secure.load(Ordering::Relaxed));
        }
    }

    #[test]
    fn test_invalid_public_settings_url_resets_secure_default() {
        let state = applied_public_settings(
            PublicSettings {
                display_password_reset: true,
                display_download_step: true,
                public_url: Some("http://edge.example.com".to_owned()),
            },
            true,
        );
        assert!(!state.cookie_secure.load(Ordering::Relaxed));

        state.apply(PublicSettings {
            display_password_reset: false,
            display_download_step: false,
            public_url: Some("not a URL".to_owned()),
        });
        assert!(state.cookie_secure.load(Ordering::Relaxed));
        assert!(!state.display_password_reset.load(Ordering::Relaxed));
        assert!(!state.display_download_step.load(Ordering::Relaxed));
    }

    /// A URL with no scheme but a port parses successfully, with the hostname taken as the
    /// scheme. Such a value must not be mistaken for a plaintext deployment.
    #[test]
    fn test_schemeless_public_settings_url_resets_secure_default() {
        for (public_url, parsed_scheme) in [
            ("edge.example.com:8443", "edge.example.com"),
            ("localhost:8080", "localhost"),
        ] {
            // Pin the premise: these parse successfully rather than erroring, so the fail-closed
            // `Err` arm never sees them and the scheme match is what has to get this right.
            assert_eq!(
                Url::parse(public_url)
                    .expect("expected a successful parse")
                    .scheme(),
                parsed_scheme
            );

            let state = applied_public_settings(
                PublicSettings {
                    display_password_reset: true,
                    display_download_step: true,
                    public_url: Some("http://edge.example.com".to_owned()),
                },
                true,
            );
            assert!(!state.cookie_secure.load(Ordering::Relaxed));

            state.apply(test_public_settings(Some(public_url)));
            assert!(
                state.cookie_secure.load(Ordering::Relaxed),
                "schemeless URL {public_url:?} must default to secure cookies"
            );
        }
    }
}
