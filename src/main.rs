use std::{fs::read_to_string, sync::Arc};

use defguard_proxy::{
    config::get_env_config,
    grpc::Configuration,
    http::{run_server, run_setup, GRPC_CERT_NAME, GRPC_KEY_NAME},
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

    let needs_setup = grpc_cert.is_none() || grpc_key.is_none();

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

    let proxy_configuration = if needs_setup {
        if let Some(logs_rx) = logs_rx {
            tracing::info!("gRPC certificates not found, running setup process");
            let proxy_configuration = run_setup(&env_config, Arc::new(Mutex::new(logs_rx))).await?;
            tracing::info!("Setup process completed successfully");
            proxy_configuration
        } else {
            anyhow::bail!(
                "gRPC certificates not found and logs receiver not available for setup process"
            );
        }
    } else if let (Some(grpc_cert), Some(grpc_key)) = (grpc_cert, grpc_key) {
        Configuration {
            grpc_cert_pem: grpc_cert,
            grpc_key_pem: grpc_key,
        }
    } else {
        anyhow::bail!("Failed to load gRPC certificates");
    };

    // run API web server
    run_server(env_config, proxy_configuration).await?;

    Ok(())
}
