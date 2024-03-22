use rust_tracing::{Event, Subscriber};
use std::fmt;
use tracing_subscriber::{
    fmt::{
        format::{self, FormatEvent, FormatFields},
        FmtContext, FormattedFields,
    },
    registry::LookupSpan,
};

pub(crate) struct HttpFormatter<E = format::Format<format::Full>> {
    inner: E,
}

impl Default for HttpFormatter {
    fn default() -> Self {
        HttpFormatter {
            inner: format::Format::default(),
        }
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
        write!(&mut writer, "HTTP_FORMATTER");
        // write!(&mut writer, "{} {}: ", metadata.level(), metadata.target())?;

        // Format all the spans in the event's span context.
        let sc = ctx.clone();
        sc.sc
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
