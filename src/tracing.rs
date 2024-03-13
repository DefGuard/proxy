use tracing::log::LevelFilter;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};


// Initializes tracing with the specified log level.
// Allows fine-grained filtering with `EnvFilter` directives.
// The directives are read from `DEFGUARD_PROXY_LOG_FILTER` env variable.
// For more info check: <https://docs.rs/tracing-subscriber/latest/tracing_subscriber/filter/struct.EnvFilter.html>
pub fn init_tracing(level: &LevelFilter) {
    tracing_subscriber::registry()
    .with(EnvFilter::try_from_env("DEFGUARD_PROXY_LOG_FILTER").unwrap_or_else(|_| {
        let filter = format!("defguard_proxy={level},tower_http={level},axum={level}", level=level);
        filter.into()
    }))
    .with(
        fmt::layer()
            // Use default field formatting, ensuring context variables are logged
            .fmt_fields(fmt::format::PrettyFields::default()),
    )
    .init();
}
