use rust_tracing::{Event, Subscriber};
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

    fn format_timestamp(&self, writer: &mut Writer<'_>) -> fmt::Result
    // where
    //     T: FormatTime,
    {
        // // If timestamps are disabled, do nothing.
        // if !self.display_timestamp {
        //     return Ok(());
        // }

        // // If ANSI color codes are enabled, format the timestamp with ANSI
        // // colors.
        // #[cfg(feature = "ansi")]
        // {
        //     if writer.has_ansi_escapes() {
        //         // let style = Style::new().dimmed();
        //         // write!(writer, "{}", style.prefix())?;

        //         // If getting the timestamp failed, don't bail --- only bail on
        //         // formatting errors.
        //         if self.timer.format_time(writer).is_err() {
        //             writer.write_str("<unknown time>")?;
        //         }

        //         write!(writer, "{} ", style.suffix())?;
        //         return Ok(());
        //     }
        // }

        // Otherwise, just format the timestamp without ANSI formatting.
        // If getting the timestamp failed, don't bail --- only bail on
        // formatting errors.
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
        // Format values from the event's's metadata:
        let metadata = event.metadata();
        self.format_timestamp(&mut writer);
        write!(&mut writer, "HTTP_FORMATTER");
        // write!(&mut writer, "{} {}: ", metadata.level(), metadata.target())?;

        // Format all the spans in the event's span context.
        let sc = ctx.clone();
        self.inner.format_event(ctx, writer, event);
        // if let Some(scope) = ctx.event_scope() {
        //     for span in scope.from_root() {
        //         write!(writer, "{}", span.name())?;

        //         // `FormattedFields` is a formatted representation of the span's
        //         // fields, which is stored in its extensions by the `fmt` layer's
        //         // `new_span` method. The fields will have been formatted
        //         // by the same field formatter that's provided to the event
        //         // formatter in the `FmtContext`.
        //         let ext = span.extensions();
        //         let fields = &ext
        //             .get::<FormattedFields<N>>()
        //             .expect("will never be `None`");

        //         // Skip formatting the fields if the span had no fields.
        //         if !fields.is_empty() {
        //             write!(writer, "{{{}}}", fields)?;
        //         }
        //         write!(writer, ": ")?;
        //     }
        // }

        // // Write fields on the event
        // ctx.field_format().format_fields(writer.by_ref(), event)?;

        // writeln!(writer)
        Ok(())
    }
}
