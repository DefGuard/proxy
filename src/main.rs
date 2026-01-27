use std::{fs::read_to_string, sync::Arc};

use defguard_proxy::{
    config::get_env_config,
    grpc::Configuration,
    http::{run_server, GRPC_CERT_NAME, GRPC_KEY_NAME},
    logging::init_tracing,
    VERSION,
};
use defguard_version::Version;
use tokio::sync::{mpsc, Mutex};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // configuration
    if dotenvy::from_filename(".env.local").is_err() {
        dotenvy::dotenv().ok();
    }

    let env_config = get_env_config()?;
    let cert_dir = env_config.cert_dir.clone();
    let (grpc_cert, grpc_key) = (
        read_to_string(cert_dir.join(GRPC_CERT_NAME)).ok(),
        read_to_string(cert_dir.join(GRPC_KEY_NAME)).ok(),
    );

    let proxy_configuration = if let (Some(grpc_cert), Some(grpc_key)) = (grpc_cert, grpc_key) {
        Some(Configuration {
            grpc_cert_pem: grpc_cert,
            grpc_key_pem: grpc_key,
        })
    } else {
        None
    };

    let needs_setup = proxy_configuration.is_none();

    // TODO: The channel size may need to be adjusted or some other approach should be used
    // to avoid dropping log messages.
    let (logs_tx, logs_rx) = if needs_setup {
        let (logs_tx, logs_rx) = mpsc::channel(200);
        (Some(logs_tx), Some(logs_rx))
    } else {
        (None, None)
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
