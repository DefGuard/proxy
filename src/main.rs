use clap::Parser;
use defguard_proxy::{config::Config, http::run_server, tracing::init_tracing};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // configuration
    if dotenvy::from_filename(".env.local").is_err() {
        dotenvy::dotenv().ok();
    }
    let config = Config::parse();
    init_tracing(&config.log_level);

    // run API web server
    run_server(config).await?;

    Ok(())
}
