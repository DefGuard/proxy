use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};


// Initializes tracing
pub fn init_tracing(level: &str) {
    tracing_subscriber::registry()
    .with(EnvFilter::try_from_env("DEFGUARD_PROXY_LOG_FILTER").unwrap_or_else(|_| {
        let filter = format!("defguard_proxy={level},tower_http={level},axum={level}", level=level);
        info!("DEFGUARD_PROXY_LOG_FILTER env variable unset or invalid, applying log filter: {}", filter);
        filter.into()
    }))
    .with(
        fmt::layer()
            // Use default field formatting, ensuring context variables are logged
            .fmt_fields(fmt::format::PrettyFields::default()),
    )
    .init();
}
