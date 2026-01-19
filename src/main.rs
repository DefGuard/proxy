use std::sync::Arc;

use defguard_proxy::{config::get_config, http::run_server, logging::init_tracing, VERSION};
use defguard_version::Version;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // configuration
    if dotenvy::from_filename(".env.local").is_err() {
        dotenvy::dotenv().ok();
    }

    // TODO: The channel size may need to be adjusted or some other approach should be used
    // to avoid dropping log messages.
    let (logs_tx, logs_rx) = mpsc::channel(200);

    let config = get_config()?;
    init_tracing(Version::parse(VERSION)?, &config.log_level, logs_tx)?;
    // read config from env
    tracing::info!("Starting ... version v{}", VERSION);

    // run API web server
    run_server(config, Arc::new(tokio::sync::Mutex::new(logs_rx))).await?;

    Ok(())
}
