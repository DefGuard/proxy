use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use defguard_proxy::server::run_server;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                "defguard_proxy=debug,tower_http=debug,axum::rejection=trace".into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // run API server
    run_server().await?;

    Ok(())
}
