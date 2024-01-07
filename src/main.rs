use clap::Parser;
use defguard_proxy::{config::Config, server::run_server};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // initialize tracing
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| {
            "defguard_proxy=debug,tower_http=debug,axum::rejection=trace".into()
        }))
        .with(fmt::layer())
        .init();

    // load .env
    if dotenvy::from_filename(".env.local").is_err() {
        dotenvy::dotenv().ok();
    }

    // read config from env
    let config = Config::parse();

    // run API server
    run_server(config).await?;

    Ok(())
}
