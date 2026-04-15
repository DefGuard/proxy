use std::{fs::read_to_string, io::ErrorKind, path::Path, sync::Arc};

use defguard_proxy::{
    VERSION,
    config::get_env_config,
    grpc::TlsConfig,
    http::{CORE_CLIENT_CERT_NAME, GRPC_CA_CERT_NAME, GRPC_CERT_NAME, GRPC_KEY_NAME, run_server},
    logging::init_tracing,
};
use defguard_version::Version;
use tokio::sync::{Mutex, mpsc};

fn read_optional_cert_file(
    file_path: &Path,
    cert_dir: &Path,
    file_label: &'static str,
) -> anyhow::Result<Option<String>> {
    match read_to_string(file_path) {
        Ok(content) => Ok(Some(content)),
        Err(err) if err.kind() == ErrorKind::NotFound => Ok(None),
        Err(err) if err.kind() == ErrorKind::PermissionDenied => anyhow::bail!(
            "Cannot access {file_label} file {}. Permission denied for certificate directory {}.",
            file_path.display(),
            cert_dir.display()
        ),
        Err(err) => {
            tracing::warn!(
                "Failed to read gRPC {file_label} at {}: {err}",
                file_path.display()
            );
            Ok(None)
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Install the aws-lc-rs CryptoProvider as the process-wide default for rustls.
    // Both aws-lc-rs and ring are pulled in transitively; without an explicit selection
    // rustls panics when it cannot determine which provider to use.
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .ok();
    // configuration
    if dotenvy::from_filename(".env.local").is_err() {
        dotenvy::dotenv().ok();
    }

    let env_config = get_env_config()?;
    let cert_dir = env_config.cert_dir.clone();
    let grpc_cert_path = cert_dir.join(GRPC_CERT_NAME);
    let grpc_key_path = cert_dir.join(GRPC_KEY_NAME);
    let grpc_ca_cert_path = cert_dir.join(GRPC_CA_CERT_NAME);
    let core_client_cert_path = cert_dir.join(CORE_CLIENT_CERT_NAME);

    let grpc_cert = read_optional_cert_file(&grpc_cert_path, &cert_dir, "certificate")?;
    let grpc_key = read_optional_cert_file(&grpc_key_path, &cert_dir, "key")?;
    let grpc_ca_cert = read_optional_cert_file(&grpc_ca_cert_path, &cert_dir, "CA certificate")?;
    let core_client_cert_pem =
        read_optional_cert_file(&core_client_cert_path, &cert_dir, "Core client certificate")?;

    let proxy_configuration =
        if let (Some(grpc_cert), Some(grpc_key), Some(grpc_ca_cert), Some(client_cert_pem)) =
            (grpc_cert, grpc_key, grpc_ca_cert, core_client_cert_pem)
        {
            let core_client_cert_der = defguard_certs::parse_pem_certificate(&client_cert_pem)
                .map_err(|e| anyhow::anyhow!("Failed to parse Core client cert: {e}"))?
                .to_vec();
            Some(TlsConfig {
                grpc_cert_pem: grpc_cert,
                grpc_key_pem: grpc_key,
                grpc_ca_cert_pem: grpc_ca_cert,
                core_client_cert_der,
            })
        } else {
            None
        };

    // TODO: The channel size may need to be adjusted or some other approach should be used
    // to avoid dropping log messages.
    let (logs_tx, logs_rx) = {
        let (logs_tx, logs_rx) = mpsc::channel(200);
        (Some(logs_tx), Some(logs_rx))
    };

    init_tracing(Version::parse(VERSION)?, &env_config.log_level, logs_tx)?;
    // read config from env
    tracing::info!("Starting ... version v{}", VERSION);

    // run API web server
    run_server(
        env_config,
        proxy_configuration,
        logs_rx.map(|r| Arc::new(Mutex::new(r))),
    )
    .await?;

    Ok(())
}
