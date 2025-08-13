use defguard_version::{
    tracing::{SpanFieldVisitor, VersionFieldLayer, VersionFilteredFields, VersionSuffixWriter},
    SystemInfo,
};
use log::LevelFilter;
use tracing::{Event, Level, Subscriber};
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

// Initializes tracing with the specified log level and version information.
// Allows fine-grained filtering with `EnvFilter` directives.
// The directives are read from `DEFGUARD_PROXY_LOG_FILTER` env variable.
// For more info read: <https://docs.rs/tracing-subscriber/latest/tracing_subscriber/filter/struct.EnvFilter.html>
pub fn init_tracing(own_version: &str, level: &LevelFilter) {
    tracing_subscriber::registry()
        .with(
            EnvFilter::try_from_env("DEFGUARD_PROXY_LOG_FILTER")
                .unwrap_or_else(|_| level.to_string().into()),
        )
        .with(VersionFieldLayer) // Add custom layer to capture span fields
        .with(
            fmt::layer()
                .event_format(HttpVersionFormatter::new(own_version, SystemInfo::get()))
                .fmt_fields(VersionFilteredFields),
        )
        .init();
    info!("Tracing initialized");
}

/// Implements fail2ban-friendly log format with version suffixes.
/// HTTP info (if available) is extracted from the specified tracing span. The format is as follows:
/// TIMESTAMP LEVEL CLIENT_ADDR METHOD URI LOG_MESSAGE [VERSION_SUFFIXES] || TRACING_DATA
pub(crate) struct HttpVersionFormatter<'a> {
    span: &'a str,
    timer: SystemTime,
    own_version: String,
    own_info: SystemInfo,
}

impl<'a> HttpVersionFormatter<'a> {
    pub fn new(own_version: &str, own_info: SystemInfo) -> Self {
        Self {
            span: "http_request",
            timer: SystemTime,
            own_version: own_version.to_string(),
            own_info,
        }
    }
}

impl HttpVersionFormatter<'_> {
    fn format_timestamp(&self, writer: &mut Writer<'_>) -> std::fmt::Result {
        if self.timer.format_time(writer).is_err() {
            writer.write_str("<unknown time>")?;
        }
        writer.write_char(' ')
    }
}

impl<S, N> FormatEvent<S, N> for HttpVersionFormatter<'_>
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        ctx: &FmtContext<'_, S, N>,
        writer: format::Writer<'_>,
        event: &Event<'_>,
    ) -> std::fmt::Result {
        let meta = event.metadata();

        // Extract version information from current span context (similar to VersionSuffixFormat)
        let mut core_version = None;
        let mut core_info = None;
        let mut proxy_version = None;
        let mut proxy_info = None;
        let mut gateway_version = None;
        let mut gateway_info = None;

        // iterate and accumulate spans storing our special span in separate variable if encountered
        let mut context_logs = String::new();
        let mut http_log: Option<String> = None;
        if let Some(scope) = ctx.event_scope() {
            let mut seen = false;
            for span in scope.from_root() {
                let span_name = span.metadata().name();
                context_logs.push_str(&format!(" {span_name}"));
                seen = true;

                // Extract version information from span extensions
                let extensions = span.extensions();
                if let Some(stored_visitor) = extensions.get::<SpanFieldVisitor>() {
                    if core_version.is_none() && stored_visitor.core_version.is_some() {
                        core_version = stored_visitor.core_version.clone();
                    }
                    if core_info.is_none() && stored_visitor.core_info.is_some() {
                        core_info = stored_visitor.core_info.clone();
                    }
                    if proxy_version.is_none() && stored_visitor.proxy_version.is_some() {
                        proxy_version = stored_visitor.proxy_version.clone();
                    }
                    if proxy_info.is_none() && stored_visitor.proxy_info.is_some() {
                        proxy_info = stored_visitor.proxy_info.clone();
                    }
                    if gateway_version.is_none() && stored_visitor.gateway_version.is_some() {
                        gateway_version = stored_visitor.gateway_version.clone();
                    }
                    if gateway_info.is_none() && stored_visitor.gateway_info.is_some() {
                        gateway_info = stored_visitor.gateway_info.clone();
                    }
                }

                if let Some(fields) = extensions.get::<FormattedFields<N>>() {
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

        // Build version suffix
        let mut version_suffix = String::new();
        let is_versioned_span =
            core_version.is_some() || proxy_version.is_some() || gateway_version.is_some();
        let is_error = *event.metadata().level() == Level::ERROR;

        if is_versioned_span || is_error {
            // Own version
            let mut own_version_str = format!(" [{}", self.own_version);
            if is_error {
                own_version_str = format!("{own_version_str} {}", self.own_info);
            }
            own_version_str = format!("{own_version_str}]");
            version_suffix.push_str(&own_version_str);
        }

        // Core version
        if let Some(ref core_version) = core_version {
            let mut core_version_str = format!("[C:{core_version}");
            if is_error {
                if let Some(ref core_info) = core_info {
                    core_version_str = format!("{core_version_str} {core_info}");
                }
            }
            core_version_str = format!("{core_version_str}]");
            version_suffix.push_str(&core_version_str);
        }

        // Proxy version
        if let Some(ref proxy_version) = proxy_version {
            let mut proxy_version_str = format!("[PX:{proxy_version}");
            if is_error {
                if let Some(ref proxy_info) = proxy_info {
                    proxy_version_str = format!("{proxy_version_str} {proxy_info}");
                }
            }
            proxy_version_str = format!("{proxy_version_str}]");
            version_suffix.push_str(&proxy_version_str);
        }

        // Gateway version
        if let Some(ref gateway_version) = gateway_version {
            let mut gateway_version_str = format!("[GW:{gateway_version}");
            if is_error {
                if let Some(ref gateway_info) = gateway_info {
                    gateway_version_str = format!("{gateway_version_str} {gateway_info}");
                }
            }
            gateway_version_str = format!("{gateway_version_str}]");
            version_suffix.push_str(&gateway_version_str);
        }

        // Create a wrapper writer that will append version info before newlines
        let mut wrapper = VersionSuffixWriter::new(writer, version_suffix);
        let mut versioned_writer = Writer::new(&mut wrapper);

        // timestamp, level & target
        self.format_timestamp(&mut versioned_writer)?;
        write!(versioned_writer, "{} ", meta.level())?;
        write!(versioned_writer, "{}: ", meta.target(),)?;

        // write http context log (ip, method, path)
        if let Some(log) = http_log {
            let split: Vec<&str> = log.split(['=', ' ']).collect();
            let method = split.get(1).unwrap_or(&"unknown");
            let path = split.get(3).unwrap_or(&"unknown");

            let addr = split.get(5).map(|s| s.replace('"', ""));
            let ip = addr
                .and_then(|s| s.split(':').next().map(ToString::to_string))
                .unwrap_or("unknown".to_string());
            write!(versioned_writer, "{ip} {method} {path} ")?;
        }

        // write actual log message
        ctx.format_fields(versioned_writer.by_ref(), event)?;

        // write span context
        if !context_logs.is_empty() {
            write!(versioned_writer, " || Tracing data: {context_logs}")?;
        }
        writeln!(versioned_writer)
    }
}
