use log::LevelFilter;
use tracing::{Event, Subscriber};
use tracing_subscriber::{
    fmt::{
        self,
        format::{self, FormatEvent, FormatFields, Writer},
        time::{FormatTime, SystemTime},
        FmtContext, FormattedFields,
    },
    layer::SubscriberExt,
    registry::LookupSpan,
    util::SubscriberInitExt,
    EnvFilter,
};

// Initializes tracing with the specified log level.
// Allows fine-grained filtering with `EnvFilter` directives.
// The directives are read from `DEFGUARD_PROXY_LOG_FILTER` env variable.
// For more info read: <https://docs.rs/tracing-subscriber/latest/tracing_subscriber/filter/struct.EnvFilter.html>
pub fn init_tracing(level: &LevelFilter) {
    tracing_subscriber::registry()
        .with(
            EnvFilter::try_from_env("DEFGUARD_PROXY_LOG_FILTER")
                .unwrap_or_else(|_| level.to_string().into()),
        )
        .with(fmt::layer().event_format(HttpFormatter::default()))
        .init();
    info!("Tracing initialized");
}

/// Implements fail2ban-friendly log format using `tracing_subscriber::fmt::format::FormatEvent` trait.
/// HTTP info (if available) is extracted from the specified tracing span. The format is as follows:
/// TIMESTAMP LEVEL CLIENT_ADDR METHOD URI LOG_MESSAGE || TRACING_DATA
pub(crate) struct HttpFormatter<'a> {
    span: &'a str,
    timer: SystemTime,
}

impl Default for HttpFormatter<'_> {
    fn default() -> Self {
        Self {
            span: "http_request",
            timer: SystemTime,
        }
    }
}

impl HttpFormatter<'_> {
    fn format_timestamp(&self, writer: &mut Writer<'_>) -> std::fmt::Result {
        if self.timer.format_time(writer).is_err() {
            writer.write_str("<unknown time>")?;
        }
        writer.write_char(' ')
    }
}

impl<S, N> FormatEvent<S, N> for HttpFormatter<'_>
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        ctx: &FmtContext<'_, S, N>,
        mut writer: format::Writer<'_>,
        event: &Event<'_>,
    ) -> std::fmt::Result {
        let meta = event.metadata();

        // timestamp, level & target
        self.format_timestamp(&mut writer)?;
        write!(writer, "{} ", meta.level())?;
        write!(writer, "{}: ", meta.target(),)?;

        // iterate and accumulate spans storing our special span in separate variable if encountered
        let mut context_logs = String::new();
        let mut http_log: Option<String> = None;
        if let Some(scope) = ctx.event_scope() {
            let mut seen = false;
            for span in scope.from_root() {
                let span_name = span.metadata().name();
                context_logs.push_str(&format!(" {span_name}"));
                seen = true;

                if let Some(fields) = span.extensions().get::<FormattedFields<N>>() {
                    if !fields.is_empty() {
                        match span_name {
                            x if x == self.span => http_log = Some(format!("{fields}")),
                            _ => context_logs.push_str(&format!(" {{{fields}}}")),
                        }
                    }
                }
                context_logs.push(':');
            }
            if seen {
                context_logs.push(' ');
            }
        };

        // write http context log (ip, method, path)
        if let Some(log) = http_log {
            let split: Vec<&str> = log.split(['=', ' ']).collect();
            let method = split.get(1).unwrap_or(&"unknown");
            let path = split.get(3).unwrap_or(&"unknown");

            let addr = split.get(5).map(|s| s.replace('"', ""));
            let ip = addr
                .and_then(|s| s.split(':').next().map(ToString::to_string))
                .unwrap_or("unknown".to_string());
            write!(writer, "{ip} {method} {path} ")?;
        }

        // write actual log message
        ctx.format_fields(writer.by_ref(), event)?;

        // write span context
        if !context_logs.is_empty() {
            write!(writer, " || Tracing data: {context_logs}")?;
        }
        writeln!(writer)
    }
}
