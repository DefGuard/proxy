use rust_tracing::{Event, Subscriber, Level};
use std::fmt;
use tracing_subscriber::{
    fmt::{
        format::{self, FormatEvent, FormatFields, Writer},
        FmtContext, FormattedFields, time::{SystemTime, FormatTime},
    },
    registry::LookupSpan,
};

#[derive(Default)]
pub(crate) struct HttpFormatter<F = format::Format<format::Full>> {
    inner: F,
    timer: SystemTime,
    display_level: bool,
}

// impl Default for HttpFormatter {
//     fn default() -> Self {
//         HttpFormatter {
//             inner: format::Format::default(),
//             timer: SystemTime::default(),
//         }
//     }
// }

impl HttpFormatter {
    fn format_timestamp(&self, writer: &mut Writer<'_>) -> fmt::Result {
        if self.timer.format_time(writer).is_err() {
            writer.write_str("<unknown time>")?;
        }
        writer.write_char(' ')
    }
}

struct FmtLevel<'a> {
    level: &'a Level,
}

impl<'a> FmtLevel<'a> {
    #[cfg(not(feature = "ansi"))]
    pub(crate) fn new(level: &'a Level) -> Self {
        Self { level }
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
        self.format_timestamp(&mut writer);
        write!(writer, "{} ", meta.level().to_string())?;

        if let Some(scope) = ctx.event_scope() {
            // let bold = writer.bold();

            let mut seen = false;

            for span in scope.from_root() {
                write!(writer, "{}", span.metadata().name())?;
                seen = true;

                let ext = span.extensions();
                if let Some(fields) = &ext.get::<FormattedFields<N>>() {
                    if !fields.is_empty() {
                        write!(writer, "{{{}}}", fields)?;
                    }
                }
                write!(writer, ":")?;
            }

            if seen {
                writer.write_char(' ')?;
            }
        };

        write!(&mut writer, "HTTP_FORMATTER END");

        // let fmt_level = FmtLevel::new(meta.level());


        self.inner.format_event(ctx, writer, event);
        Ok(())
    }
}
