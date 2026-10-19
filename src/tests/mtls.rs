use std::{
    env::temp_dir,
    net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener},
    sync::{
        Arc, RwLock,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

use axum_extra::extract::cookie::Key;
use defguard_certs::{
    CertificateAuthority, Csr, PemLabel, cert_der_to_pem, der_to_pem, generate_key_pair,
};
use futures_util::stream;
use rustls::crypto::aws_lc_rs;
use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt},
    net::{TcpListener as AsyncTcpListener, TcpStream},
    spawn,
    sync::{Mutex, broadcast, mpsc, oneshot},
    time::{Instant, sleep},
};
use tonic::{
    Code, Request, Status,
    service::Interceptor,
    transport::{Certificate, Channel, ClientTlsConfig, Endpoint, Identity},
};

use crate::{
    config::{DEFAULT_GRPC_KEEPALIVE_INTERVAL_SECONDS, DEFAULT_GRPC_KEEPALIVE_TIMEOUT_SECONDS},
    grpc::{ProxyServer, TlsConfig},
    proto::proxy_client::ProxyClient,
};

struct TestCerts {
    /// PEM-encoded CA certificate (used as the trust root for both server and client validation).
    ca_cert_pem: String,
    /// PEM-encoded proxy gRPC server certificate (ServerAuth EKU, IP SAN 127.0.0.1).
    proxy_cert_pem: String,
    /// PEM-encoded proxy gRPC server private key.
    proxy_key_pem: String,
    /// DER-encoded Core client certificate (serial A - matches what the server pins).
    core_client_cert_der: Vec<u8>,
    /// PEM-encoded Core client certificate (serial A).
    core_client_cert_pem: String,
    /// PEM-encoded Core client private key (serial A).
    core_client_key_pem: String,
    /// PEM-encoded client cert with serial B - valid CA but different serial.
    wrong_serial_cert_pem: String,
    /// PEM-encoded private key for the serial-B cert.
    wrong_serial_key_pem: String,
    /// PEM-encoded client cert signed by a completely different (rogue) CA.
    rogue_client_cert_pem: String,
    /// PEM-encoded private key for the rogue cert.
    rogue_client_key_pem: String,
}

impl TestCerts {
    fn generate() -> Self {
        // Trust-anchor CA
        let ca = CertificateAuthority::new("Test CA", "test@test.local", 365).unwrap();
        let ca_cert_pem = ca.cert_pem().unwrap();

        // Proxy server cert: ServerAuth EKU, IP SAN 127.0.0.1
        let proxy_key = generate_key_pair().unwrap();
        let proxy_csr = Csr::new(&proxy_key, &["127.0.0.1".to_string()], Vec::new()).unwrap();
        let proxy_server_cert = ca.sign_server_cert(&proxy_csr).unwrap();
        let proxy_cert_pem = cert_der_to_pem(proxy_server_cert.der()).unwrap();
        let proxy_key_pem = der_to_pem(proxy_key.serialized_der(), PemLabel::PrivateKey).unwrap();

        // Core client cert A - the "good" serial that the server will pin
        let client_a = ca.issue_core_client_cert("core-client-a").unwrap();
        let core_client_cert_der = client_a.cert_der.clone();
        let core_client_cert_pem = cert_der_to_pem(&client_a.cert_der).unwrap();
        let core_client_key_pem = der_to_pem(&client_a.key_der, PemLabel::PrivateKey).unwrap();

        // Core client cert B - different cert (different serial) but same CA
        let client_b = ca.issue_core_client_cert("core-client-b").unwrap();
        let wrong_serial_cert_pem = cert_der_to_pem(&client_b.cert_der).unwrap();
        let wrong_serial_key_pem = der_to_pem(&client_b.key_der, PemLabel::PrivateKey).unwrap();

        // Rogue CA + client cert - different trust chain entirely
        let rogue_ca = CertificateAuthority::new("Rogue CA", "rogue@rogue.local", 365).unwrap();
        let rogue_client = rogue_ca.issue_core_client_cert("rogue-core").unwrap();
        let rogue_client_cert_pem = cert_der_to_pem(&rogue_client.cert_der).unwrap();
        let rogue_client_key_pem = der_to_pem(&rogue_client.key_der, PemLabel::PrivateKey).unwrap();

        Self {
            ca_cert_pem,
            proxy_cert_pem,
            proxy_key_pem,
            core_client_cert_der,
            core_client_cert_pem,
            core_client_key_pem,
            wrong_serial_cert_pem,
            wrong_serial_key_pem,
            rogue_client_cert_pem,
            rogue_client_key_pem,
        }
    }
}

fn make_tls_config(certs: &TestCerts) -> TlsConfig {
    TlsConfig {
        grpc_key_pem: certs.proxy_key_pem.clone(),
        grpc_cert_pem: certs.proxy_cert_pem.clone(),
        grpc_ca_cert_pem: certs.ca_cert_pem.clone(),
        core_client_cert_der: certs.core_client_cert_der.clone(),
    }
}

fn build_proxy_server() -> ProxyServer {
    let (reset_tx, _) = broadcast::channel(1);
    let (https_cert_tx, _) = broadcast::channel(1);
    let (clear_https_tx, _) = broadcast::channel(1);
    let (_, logs_rx) = mpsc::channel(1);
    let cookie_key = Arc::new(RwLock::new(Some(Key::generate())));
    ProxyServer::new(
        cookie_key,
        temp_dir(),
        reset_tx,
        https_cert_tx,
        clear_https_tx,
        None,
        Arc::new(Mutex::new(logs_rx)),
        false,
    )
}

/// Install the rustls AWS-LC crypto provider for the process.
///
/// Must be called before any TLS code runs. Safe to call from multiple tests -
/// subsequent calls after the first succeed-or-fail are silently ignored.
fn init_crypto() {
    let _ = aws_lc_rs::default_provider().install_default();
}

/// Spawn a configured `ProxyServer` on an OS-assigned port.
///
/// Returns `(bound_addr, shutdown_tx, connected)`. Send `shutdown_tx` to stop the server.
///
/// Waits until the server is accepting TCP connections before returning, so
/// callers do not need a fixed sleep to avoid startup races.
async fn spawn_test_proxy(certs: &TestCerts) -> (SocketAddr, oneshot::Sender<()>, Arc<AtomicBool>) {
    let server = build_proxy_server();
    let connected = Arc::clone(&server.connected);
    server.configure(make_tls_config(certs));

    // Find a free port, drop the listener, pass the addr to run().
    // The small race window is acceptable in test context.
    let addr = TcpListener::bind(SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0))
        .unwrap()
        .local_addr()
        .unwrap();

    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
    spawn(async move {
        let _ = server
            .run(
                addr,
                Duration::from_secs(DEFAULT_GRPC_KEEPALIVE_INTERVAL_SECONDS.get()),
                Duration::from_secs(DEFAULT_GRPC_KEEPALIVE_TIMEOUT_SECONDS.get()),
                async move {
                    let _ = shutdown_rx.await;
                },
            )
            .await;
    });

    // Wait until the server is accepting TCP connections instead of sleeping a
    // fixed amount. This eliminates the flaky startup race while keeping the
    // helper fast on capable hardware.
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        assert!(
            Instant::now() < deadline,
            "timeout waiting for test gRPC server to start on {addr}"
        );
        if TcpStream::connect(addr).await.is_ok() {
            break;
        }
        sleep(Duration::from_millis(20)).await;
    }

    (addr, shutdown_tx, connected)
}

async fn relay_bytes<R, W>(
    mut reader: R,
    mut writer: W,
    forward_bytes: Arc<AtomicBool>,
) -> tokio::io::Result<()>
where
    R: AsyncRead + Unpin,
    W: AsyncWrite + Unpin,
{
    let mut buffer = [0; 8192];
    loop {
        let bytes_read = reader.read(&mut buffer).await?;
        if bytes_read == 0 {
            return Ok(());
        }
        if forward_bytes.load(Ordering::Relaxed) {
            writer.write_all(&buffer[..bytes_read]).await?;
        }
    }
}

async fn spawn_test_relay(
    target: SocketAddr,
) -> (SocketAddr, Arc<AtomicBool>, tokio::task::JoinHandle<()>) {
    let listener = AsyncTcpListener::bind(SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0))
        .await
        .expect("Failed to bind test relay");
    let addr = listener.local_addr().expect("Failed to get relay address");
    let forward_bytes = Arc::new(AtomicBool::new(true));
    let relay_forward_bytes = Arc::clone(&forward_bytes);
    let relay_task = spawn(async move {
        let (client, _) = listener
            .accept()
            .await
            .expect("Failed to accept relay client");
        let upstream = TcpStream::connect(target)
            .await
            .expect("Failed to connect relay to test server");
        let (client_read, client_write) = client.into_split();
        let (upstream_read, upstream_write) = upstream.into_split();
        let client_to_server = relay_bytes(
            client_read,
            upstream_write,
            Arc::clone(&relay_forward_bytes),
        );
        let server_to_client = relay_bytes(upstream_read, client_write, relay_forward_bytes);
        let result = tokio::select! {
            result = client_to_server => result,
            result = server_to_client => result,
        };
        assert!(result.is_ok(), "TCP relay failed: {result:?}");
    });

    (addr, forward_bytes, relay_task)
}

/// Build a tonic `ProxyClient` using the given CA and optional client identity.
///
/// `client_identity` is `Some((cert_pem, key_pem))` for mTLS; `None` for no client cert.
async fn connect(
    addr: SocketAddr,
    ca_cert_pem: &str,
    client_identity: Option<(&str, &str)>,
) -> Result<ProxyClient<Channel>, tonic::transport::Error> {
    let mut tls = ClientTlsConfig::new().ca_certificate(Certificate::from_pem(ca_cert_pem));

    if let Some((cert_pem, key_pem)) = client_identity {
        tls = tls.identity(Identity::from_pem(cert_pem, key_pem));
    }

    let channel = Endpoint::from_shared(format!("https://127.0.0.1:{}", addr.port()))
        .unwrap()
        .tls_config(tls)?
        .connect()
        .await?;

    Ok(ProxyClient::new(channel))
}

/// Open a `bidi` streaming call with an empty request stream and return the status code.
///
/// The stream body is irrelevant - we only care whether the mTLS + serial-pin interceptors
/// accept or reject the connection.
async fn call_bidi(client: &mut ProxyClient<Channel>) -> Status {
    let empty = Vec::new();
    match client.bidi(Request::new(stream::iter(empty))).await {
        Ok(_) => Status::ok("accepted"),
        Err(status) => status,
    }
}

/// `run()` must return `Err` immediately when no `TlsConfig` has been set.
#[tokio::test]
async fn run_errors_without_tls_config() {
    let server = build_proxy_server();
    // configure() is deliberately NOT called.
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0);
    let result = server
        .run(
            addr,
            Duration::from_secs(DEFAULT_GRPC_KEEPALIVE_INTERVAL_SECONDS.get()),
            Duration::from_secs(DEFAULT_GRPC_KEEPALIVE_TIMEOUT_SECONDS.get()),
            futures_util::future::pending::<()>(),
        )
        .await;
    assert!(result.is_err(), "expected Err, got Ok");
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("TLS configuration is missing"),
        "unexpected error message",
    );
}

/// A client presenting the correct CA-signed cert with the expected serial must be accepted.
///
/// The `bidi` call may be rejected by the version interceptor (no version headers are sent),
/// but it must NOT be rejected with `Unauthenticated` - that would indicate the mTLS layer
/// or serial-pin interceptor wrongly rejected the cert.
#[tokio::test]
async fn valid_mtls_client_accepted() {
    init_crypto();
    let certs = TestCerts::generate();
    let (addr, shutdown_tx, _) = spawn_test_proxy(&certs).await;

    let mut client = connect(
        addr,
        &certs.ca_cert_pem,
        Some((&certs.core_client_cert_pem, &certs.core_client_key_pem)),
    )
    .await
    .expect("TLS handshake should succeed with valid client cert");

    let status = call_bidi(&mut client).await;

    assert_ne!(
        status.code(),
        Code::Unauthenticated,
        "valid client cert should not be rejected; got: {status}",
    );

    let _ = shutdown_tx.send(());
}

#[tokio::test]
#[ignore = "takes about 30 seconds; run explicitly for keepalive coverage"]
async fn test_silent_bidi_connection_drop_is_detected_by_keepalive() {
    init_crypto();
    let certs = TestCerts::generate();
    let (server_addr, shutdown_tx, connected) = spawn_test_proxy(&certs).await;
    let (relay_addr, forward_bytes, relay_task) = spawn_test_relay(server_addr).await;
    let mut client = connect(
        relay_addr,
        &certs.ca_cert_pem,
        Some((&certs.core_client_cert_pem, &certs.core_client_key_pem)),
    )
    .await
    .expect("TLS handshake should succeed with valid client cert");

    let mut interceptor =
        defguard_version::client::ClientVersionInterceptor::new(crate::MIN_CORE_VERSION.clone());
    let version_headers = interceptor
        .call(Request::new(()))
        .expect("Core version headers should be valid");
    let mut request = Request::new(stream::pending::<crate::proto::CoreResponse>());
    *request.metadata_mut() = version_headers.metadata().clone();
    let response = client
        .bidi(request)
        .await
        .expect("valid Core client should open a bidi stream");
    assert!(connected.load(Ordering::Relaxed));

    forward_bytes.store(false, Ordering::Relaxed);
    tokio::time::timeout(Duration::from_secs(35), async {
        while connected.load(Ordering::Relaxed) {
            sleep(Duration::from_millis(100)).await;
        }
    })
    .await
    .expect("Edge did not detect the silently dropped Core within 35 seconds");

    let _ = shutdown_tx.send(());
    drop(response);
    drop(client);
    relay_task.await.expect("test relay task failed");
}

/// A client that presents no certificate must be rejected at the TLS layer.
#[tokio::test]
async fn no_client_cert_rejected() {
    init_crypto();
    let certs = TestCerts::generate();
    let (addr, shutdown_tx, _) = spawn_test_proxy(&certs).await;

    // spawn_test_proxy guarantees the server is ready, so a connect() failure
    // here is a genuine TLS rejection rather than a startup race.
    let rejected = match connect(addr, &certs.ca_cert_pem, None).await {
        Err(_) => true, // eager TLS rejection at connect time
        Ok(mut client) => {
            // Lazy path: TLS handshake occurs on the first RPC.
            client
                .bidi(Request::new(stream::iter(Vec::new())))
                .await
                .is_err()
        }
    };

    assert!(
        rejected,
        "connecting without a client cert should be rejected",
    );

    let _ = shutdown_tx.send(());
}

/// A client presenting a cert from the correct CA but with the wrong serial must be rejected
/// by the serial-pin interceptor with `Unauthenticated`.
#[tokio::test]
async fn wrong_serial_rejected() {
    init_crypto();
    let certs = TestCerts::generate();
    let (addr, shutdown_tx, _) = spawn_test_proxy(&certs).await;

    // This cert is valid (signed by the CA the server trusts) but has a different serial.
    let mut client = connect(
        addr,
        &certs.ca_cert_pem,
        Some((&certs.wrong_serial_cert_pem, &certs.wrong_serial_key_pem)),
    )
    .await
    .expect("TLS handshake should succeed; the serial check runs as a gRPC interceptor");

    let status = call_bidi(&mut client).await;

    assert_eq!(
        status.code(),
        Code::Unauthenticated,
        "wrong-serial cert must be rejected with Unauthenticated; got: {status}",
    );

    let _ = shutdown_tx.send(());
}

/// A client presenting a cert signed by a different (rogue) CA must be rejected at the TLS
/// layer because the server does not trust that CA.
///
/// tonic may reject the connection eagerly (at `connect()` time) or lazily
/// (on the first RPC). Both outcomes are treated as the expected rejection,
/// but the rejection must not be `FailedPrecondition`, which would indicate
/// the cert bypassed CA verification and reached the gRPC handler.
#[tokio::test]
async fn rogue_ca_client_rejected() {
    init_crypto();
    let certs = TestCerts::generate();
    let (addr, shutdown_tx, _) = spawn_test_proxy(&certs).await;

    // spawn_test_proxy guarantees the server is ready, so a connect() failure
    // here is a genuine TLS rejection rather than a startup race.
    let rpc_status = match connect(
        addr,
        &certs.ca_cert_pem,
        Some((&certs.rogue_client_cert_pem, &certs.rogue_client_key_pem)),
    )
    .await
    {
        Err(_) => {
            // Eager TLS rejection at connect time - correct behavior.
            let _ = shutdown_tx.send(());
            return;
        }
        Ok(mut client) => {
            // Lazy path: TLS handshake occurs on the first RPC.
            match client.bidi(Request::new(stream::iter(Vec::new()))).await {
                Ok(_) => {
                    let _ = shutdown_tx.send(());
                    panic!("rogue-CA client cert must be rejected; got Ok");
                }
                Err(status) => status,
            }
        }
    };

    // Must NOT be FailedPrecondition - that would mean the cert was accepted by the
    // TLS layer and reached the gRPC handler.
    assert_ne!(
        rpc_status.code(),
        Code::FailedPrecondition,
        "rogue-CA cert reached the gRPC handler - server-side CA verification is missing; \
         got: {rpc_status}",
    );

    let _ = shutdown_tx.send(());
}
