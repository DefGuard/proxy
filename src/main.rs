use std::{fs::read_to_string, io::ErrorKind, path::Path, sync::Arc};

use defguard_proxy::{
    VERSION,
    config::get_env_config,
    grpc::Configuration,
    http::{GRPC_CERT_NAME, GRPC_KEY_NAME, run_server},
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
    // configuration
    if dotenvy::from_filename(".env.local").is_err() {
        dotenvy::dotenv().ok();
    }

    let env_config = get_env_config()?;
    let cert_dir = env_config.cert_dir.clone();
    let grpc_cert_path = cert_dir.join(GRPC_CERT_NAME);
    let grpc_key_path = cert_dir.join(GRPC_KEY_NAME);

    let grpc_cert = read_optional_cert_file(&grpc_cert_path, &cert_dir, "certificate")?;
    let grpc_key = read_optional_cert_file(&grpc_key_path, &cert_dir, "key")?;

    let proxy_configuration = if let (Some(grpc_cert), Some(grpc_key)) = (grpc_cert, grpc_key) {
        Some(Configuration {
            grpc_cert_pem: grpc_cert,
            grpc_key_pem: grpc_key,
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
