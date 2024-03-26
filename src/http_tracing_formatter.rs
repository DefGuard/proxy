use rust_tracing::{Event, Subscriber};
use std::fmt;
use tracing_subscriber::{
    fmt::{
        format::{self, FormatEvent, FormatFields, Writer},
        time::{FormatTime, SystemTime},
        FmtContext, FormattedFields,
    },
    registry::LookupSpan,
};

const HTTP_SPAN_NAME: &str = "http_request";

#[derive(Default)]
pub(crate) struct HttpFormatter {
    timer: SystemTime,
}

impl HttpFormatter {
    fn format_timestamp(&self, writer: &mut Writer<'_>) -> fmt::Result {
        if self.timer.format_time(writer).is_err() {
            writer.write_str("<unknown time>")?;
        }
        writer.write_char(' ')
    }
}

impl<S, N> FormatEvent<S, N> for HttpFormatter
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        ctx: &FmtContext<'_, S, N>,
        mut writer: format::Writer<'_>,
        event: &Event<'_>,
    ) -> fmt::Result {
        let meta = event.metadata();

        // Format values from the event's's metadata:
        self.format_timestamp(&mut writer)?;
        write!(writer, "{} ", meta.level().to_string())?;

        let mut logs: Vec<String> = Vec::new();
        let mut http_logs: Vec<String> = Vec::new();
        if let Some(scope) = ctx.event_scope() {
            let mut seen = false;
            for span in scope.from_root() {
                let span_name = span.metadata().name();
                logs.push(format!("{}", span_name));
                seen = true;

                if let Some(fields) = span.extensions().get::<FormattedFields<N>>() {
                    if !fields.is_empty() {
                        if span_name == HTTP_SPAN_NAME {
                            http_logs.push(format!("{}", fields));
                            continue;
                        }
                        logs.push(format!("{{{}}}", fields));
                    }
                }
                logs.push(":".into());
            }

            if seen {
                logs.push(' '.into());
            }
        };

        for log in http_logs {
            let split: Vec<&str> = log.split(['=', ' ']).collect();
            let method = split.get(1).unwrap_or(&"unknown");
            let path = split.get(3).unwrap_or(&"unknown");
            let ip = split.get(5).unwrap_or(&"unknown").replace('"', "");
            write!(writer, "{} {} {} ", ip, method, path)?;
        }
        for log in logs {
            write!(writer, "{}", log)?
        }

        // self.inner.format_event(ctx, writer, event)?;
        ctx.format_fields(writer.by_ref(), event)?;
        writeln!(writer)
    }
}
