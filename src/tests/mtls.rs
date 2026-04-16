use std::{
    env::temp_dir,
    net::{SocketAddr, TcpListener},
    sync::{Arc, RwLock},
    time::Duration,
};

use axum_extra::extract::cookie::Key;
use defguard_certs::{
    CertificateAuthority, Csr, PemLabel, cert_der_to_pem, der_to_pem, generate_key_pair,
};
use futures_util::stream;
use rustls::crypto::aws_lc_rs;
use tokio::{
    spawn,
    sync::{Mutex, broadcast, mpsc, oneshot},
    time::sleep,
};
use tonic::{
    Code, Request, Status,
    transport::{Certificate, Channel, ClientTlsConfig, Endpoint, Identity},
};

use crate::grpc::{ProxyServer, TlsConfig};
use crate::proto::{CoreResponse, proxy_client::ProxyClient};

struct TestCerts {
    /// PEM-encoded CA certificate (used as the trust root for both server and client validation).
    ca_cert_pem: String,
    /// PEM-encoded proxy gRPC server certificate (ServerAuth EKU, IP SAN 127.0.0.1).
    proxy_cert_pem: String,
    /// PEM-encoded proxy gRPC server private key.
    proxy_key_pem: String,
    /// DER-encoded Core client certificate (serial A — matches what the server pins).
    core_client_cert_der: Vec<u8>,
    /// PEM-encoded Core client certificate (serial A).
    core_client_cert_pem: String,
    /// PEM-encoded Core client private key (serial A).
    core_client_key_pem: String,
    /// PEM-encoded client cert with serial B — valid CA but different serial.
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
        let proxy_csr = Csr::new(&proxy_key, &["127.0.0.1".to_string()], vec![]).unwrap();
        let proxy_server_cert = ca.sign_server_cert(&proxy_csr).unwrap();
        let proxy_cert_pem = cert_der_to_pem(proxy_server_cert.der()).unwrap();
        let proxy_key_pem = der_to_pem(proxy_key.serialized_der(), PemLabel::PrivateKey).unwrap();

        // Core client cert A — the "good" serial that the server will pin
        let client_a = ca.issue_core_client_cert("core-client-a").unwrap();
        let core_client_cert_der = client_a.cert_der.clone();
        let core_client_cert_pem = cert_der_to_pem(&client_a.cert_der).unwrap();
        let core_client_key_pem = der_to_pem(&client_a.key_der, PemLabel::PrivateKey).unwrap();

        // Core client cert B — different cert (different serial) but same CA
        let client_b = ca.issue_core_client_cert("core-client-b").unwrap();
        let wrong_serial_cert_pem = cert_der_to_pem(&client_b.cert_der).unwrap();
        let wrong_serial_key_pem = der_to_pem(&client_b.key_der, PemLabel::PrivateKey).unwrap();

        // Rogue CA + client cert — different trust chain entirely
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
/// Must be called before any TLS code runs. Safe to call from multiple tests —
/// subsequent calls after the first succeed-or-fail are silently ignored.
fn init_crypto() {
    let _ = aws_lc_rs::default_provider().install_default();
}

/// Spawn a configured `ProxyServer` on an OS-assigned port.
///
/// Returns `(bound_addr, shutdown_tx)`. Drop / send `shutdown_tx` to stop the server.
async fn spawn_test_proxy(certs: &TestCerts) -> (SocketAddr, oneshot::Sender<()>) {
    let server = build_proxy_server();
    server.configure(make_tls_config(certs));

    // Find a free port, drop the listener, pass the addr to run().
    // The small race window is acceptable in test context.
    let addr = TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap();

    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
    spawn(async move {
        let _ = server
            .run(addr, async move {
                let _ = shutdown_rx.await;
            })
            .await;
    });

    // Give tonic time to bind and start serving.
    sleep(Duration::from_millis(150)).await;

    (addr, shutdown_tx)
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
/// The stream body is irrelevant — we only care whether the mTLS + serial-pin interceptors
/// accept or reject the connection.
async fn call_bidi(client: &mut ProxyClient<Channel>) -> Status {
    let empty: Vec<CoreResponse> = vec![];
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
    let addr: SocketAddr = "127.0.0.1:0".parse().unwrap();
    let result = server
        .run(addr, futures_util::future::pending::<()>())
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
/// but it must NOT be rejected with `Unauthenticated` — that would indicate the mTLS layer
/// or serial-pin interceptor wrongly rejected the cert.
#[tokio::test]
async fn valid_mtls_client_accepted() {
    init_crypto();
    let certs = TestCerts::generate();
    let (addr, shutdown_tx) = spawn_test_proxy(&certs).await;

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

/// A client that presents no certificate must be rejected at the TLS layer.
#[tokio::test]
async fn no_client_cert_rejected() {
    init_crypto();
    let certs = TestCerts::generate();
    let (addr, shutdown_tx) = spawn_test_proxy(&certs).await;

    // connect() is lazy in tonic — it doesn't perform the TLS handshake until the first RPC.
    // We must make an RPC call to actually trigger the handshake and observe the rejection.
    let Ok(mut client) = connect(addr, &certs.ca_cert_pem, None).await else {
        // If connect() fails eagerly, that also counts as rejection.
        let _ = shutdown_tx.send(());
        return;
    };

    let empty: Vec<CoreResponse> = vec![];
    let result = client.bidi(Request::new(stream::iter(empty))).await;

    assert!(
        result.is_err(),
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
    let (addr, shutdown_tx) = spawn_test_proxy(&certs).await;

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
#[tokio::test]
async fn rogue_ca_client_rejected() {
    init_crypto();
    let certs = TestCerts::generate();
    let (addr, shutdown_tx) = spawn_test_proxy(&certs).await;

    // connect() is lazy in tonic — the TLS handshake happens on the first RPC.
    let Ok(mut client) = connect(
        addr,
        &certs.ca_cert_pem,
        Some((&certs.rogue_client_cert_pem, &certs.rogue_client_key_pem)),
    )
    .await
    else {
        // Eager rejection also counts.
        let _ = shutdown_tx.send(());
        return;
    };

    let empty: Vec<CoreResponse> = vec![];
    let result = client.bidi(Request::new(stream::iter(empty))).await;

    assert!(
        result.is_err(),
        "rogue-CA client cert must be rejected; got Ok",
    );
    // Must NOT be a successful gRPC-level response — the error must be transport-level or
    // Unauthenticated, not FailedPrecondition (which would indicate the cert was accepted).
    if let Err(ref status) = result {
        assert_ne!(
            status.code(),
            Code::FailedPrecondition,
            "rogue-CA cert reached the gRPC handler — server-side CA verification is missing; \
             got: {status}",
        );
    }

    let _ = shutdown_tx.send(());
}
