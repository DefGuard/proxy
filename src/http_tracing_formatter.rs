use rust_tracing::{Event, Subscriber};
use std::fmt;
use tracing_subscriber::{
    fmt::{
        format::{self, FormatEvent, FormatFields},
        FmtContext, FormattedFields,
    },
    registry::LookupSpan,
};

// pub(crate) struct HttpFormatter<S, N = format::DefaultFields, E = format::Format<format::Full>> {
pub(crate) struct HttpFormatter<E = format::Format<format::Full>> {
    inner: E,
}

impl Default for HttpFormatter {
    fn default() -> Self {
        // only enable ANSI when the feature is enabled, and the NO_COLOR
        // environment variable is unset or empty.
        // let ansi = cfg!(feature = "ansi") && env::var("NO_COLOR").map_or(true, |v| v.is_empty());

        HttpFormatter {
            // fmt_fields: format::DefaultFields::default(),
            inner: format::Format::default(),
            // fmt_span: format::FmtSpanConfig::default(),
            // make_writer: io::stdout,
            // is_ansi: ansi,
            // log_internal_errors: false,
            // _inner: PhantomData,
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
        write!(&mut writer, "{} {}: ", metadata.level(), metadata.target())?;

        // Format all the spans in the event's span context.
        if let Some(scope) = ctx.event_scope() {
            for span in scope.from_root() {
                write!(writer, "{}", span.name())?;

                // `FormattedFields` is a formatted representation of the span's
                // fields, which is stored in its extensions by the `fmt` layer's
                // `new_span` method. The fields will have been formatted
                // by the same field formatter that's provided to the event
                // formatter in the `FmtContext`.
                let ext = span.extensions();
                let fields = &ext
                    .get::<FormattedFields<N>>()
                    .expect("will never be `None`");

                // Skip formatting the fields if the span had no fields.
                if !fields.is_empty() {
                    write!(writer, "{{{}}}", fields)?;
                }
                write!(writer, ": ")?;
            }
        }

        // Write fields on the event
        ctx.field_format().format_fields(writer.by_ref(), event)?;

        writeln!(writer)
    }
}

// let _subscriber = tracing_subscriber::fmt()
//     .event_format(MyFormatter)
//     .init();

// let _span = tracing::info_span!("my_span", answer = 42).entered();
// tracing::info!(question = "life, the universe, and everything", "hello world");


