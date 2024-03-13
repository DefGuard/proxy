use clap::Parser;
use defguard_proxy::{config::Config, server::run_server};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // configuration
    if dotenvy::from_filename(".env.local").is_err() {
        dotenvy::dotenv().ok();
    }
    let config = Config::parse();

    // initialize tracing
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| {
            format!(
                "defguard_proxy={level},tower_http={level},axum={level}",
                level = config.log_level,
            )
            .into()
        }))
        .with(
            fmt::layer()
                // Include span context with each log message
                .with_span_events(fmt::format::FmtSpan::NEW)
                // Use default field formatting, ensuring context variables are logged
                .fmt_fields(fmt::format::PrettyFields::default()),
        )
        // // Enable full logging of spans, including entering, exiting, and closing spans
        // .with_span_events(fmt::format::FmtSpan::FULL)
        // // // Optionally, customize the format to include more span information
        // .fmt_fields(fmt::format::debug)
        .init();

    // run API server
    run_server(config).await?;

    Ok(())
}
