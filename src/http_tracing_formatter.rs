use rust_tracing::{field::Field, Event, Level, Subscriber};
use std::fmt;
use tracing_subscriber::{
    fmt::{
        format::{self, FormatEvent, FormatFields, Writer},
        time::{FormatTime, SystemTime},
        FmtContext, FormattedFields,
    },
    registry::{LookupSpan, Scope},
};

const HTTP_SPAN_NAME: &str = "http_request";

#[derive(Default)]
pub(crate) struct HttpFormatter<F = format::Format<format::Full>> {
    inner: F,
    timer: SystemTime,
}

impl HttpFormatter {
    fn format_timestamp(&self, writer: &mut Writer<'_>) -> fmt::Result {
        if self.timer.format_time(writer).is_err() {
            writer.write_str("<unknown time>")?;
        }
        writer.write_char(' ')
    }

    // // Checks if http_request span is present in the scope
    // // If present, logs scope variables in fail2ban-friendly format
    // fn write_http_span<S: Subscriber + for<'a> LookupSpan<'a>>(&self, scope: &Scope<S>) {
    //     let span = scope.cloned().from_root();
    // }
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
        // let mut http_fields: Option<&FormattedFields<N>> = None;
        if let Some(scope) = ctx.event_scope() {
            // let http_span = scope.from_root().filter(|span| span.metadata().name() == HTTP_SPAN_NAME);
            let mut seen = false;
            // let ext;
            for span in scope.from_root() {
                // write!(writer, "{}", span.metadata().name())?;
                let span_name = span.metadata().name();
                logs.push(format!("{}", span_name));
                seen = true;

                // let ext = span.extensions();
                // let fields = ext.get::<FormattedFields<N>>();
                // span.get(&tracing::field::Field::display(HTTP_SPAN_NAME));
                if let Some(fields) = span.extensions().get::<FormattedFields<N>>() {
                    if !fields.is_empty() {
                        // write!(writer, "{{{}}}", fields)?;
                        if span_name == HTTP_SPAN_NAME {
                            http_logs.push(format!("{}", fields));
                            continue;
                        }
                        logs.push(format!("{{{}}}", fields));
                    }
                }
                // write!(writer, ":")?;
                logs.push(":".into());
            }

            if seen {
                // writer.write_char(' ')?;
                logs.push(' '.into());
            }
        };

        for log in http_logs {
            write!(writer, "HTTP_LINE: {}", log)?;
            let mut split = log.split(['=', ' ']);
            split.next();
            let fields = split.step_by(2).map(|s| s.trim()).collect::<Vec<&str>>().join(" ");
            write!(writer, "HTTP: {}", fields)?
        }
        for log in logs {
            write!(writer, "{}", log)?
        }

        write!(&mut writer, "HTTP_FORMATTER END")?;

        self.inner.format_event(ctx, writer, event)?;
        Ok(())
    }
}
