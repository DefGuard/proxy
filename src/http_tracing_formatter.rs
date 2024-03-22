use rust_tracing::{Event, Subscriber};
use std::fmt;
// use tracing_core::{Subscriber, Event};
use tracing_subscriber::{
    fmt::{
        format::{self, FormatEvent, FormatFields, Format, Full},
        FmtContext, FormattedFields, time::SystemTime, Layer,
    },
    registry::LookupSpan,
};

// pub(crate) struct HttpFormatter<S, N> {
//     // inner: Box<dyn FormatEvent<S, N>>,
//     inner: Box<dyn FormatEvent<S, N>>,
// }

// impl<S, N> Default for HttpFormatter<S, N> {
//     fn default() -> Self {
//         Self {
//             inner: Box::new(Format::default())
//         }
//     }
// }

// Assuming `Full` is the desired inner formatter
pub(crate) struct HttpFormatter<N> {
    // inner: Format<Full, N>, // Adjusted to use `Format` with `Full`
    inner: Layer<Subscriber>,
}

// impl<N> Default for HttpFormatter<N> {
//     fn default() -> Self {
//         Self {
//             inner: Format::default(),
//         }
//     }
// }

impl<N> Default for HttpFormatter<N>
where
    N: for<'a> FormatFields<'a> + 'static,
{
    fn default() -> Self {
        Self {
            // Properly instantiate the inner formatter with the Full format
            // inner: Format::new(format::Full::default()), 
            inner: Format::new(format::Full::default()), 
        }
    }
}

impl<S, N> FormatEvent<S, N> for HttpFormatter<N>
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
// impl<S, N> FormatEvent<S, N> for HttpFormatter<S, N>
// where
//     S: Subscriber + for<'a> LookupSpan<'a>,
//     N: for<'a> FormatFields<'a> + 'static,
// {
//     fn format_event(
//         &self,
//         ctx: &FmtContext<'_, S, N>,
//         mut writer: format::Writer<'_>,
//         event: &Event<'_>,
//     ) -> fmt::Result {
//         // Format values from the event's's metadata:
//         let metadata = event.metadata();
//         write!(&mut writer, "{} {}: ", metadata.level(), metadata.target())?;

//         // Format all the spans in the event's span context.
//         if let Some(scope) = ctx.event_scope() {
//             for span in scope.from_root() {
//                 write!(writer, "{}", span.name())?;

//                 // `FormattedFields` is a formatted representation of the span's
//                 // fields, which is stored in its extensions by the `fmt` layer's
//                 // `new_span` method. The fields will have been formatted
//                 // by the same field formatter that's provided to the event
//                 // formatter in the `FmtContext`.
//                 let ext = span.extensions();
//                 let fields = &ext
//                     .get::<FormattedFields<N>>()
//                     .expect("will never be `None`");

//                 // Skip formatting the fields if the span had no fields.
//                 if !fields.is_empty() {
//                     write!(writer, "{{{}}}", fields)?;
//                 }
//                 write!(writer, ": ")?;
//             }
//         }

//         // Write fields on the event
//         ctx.field_format().format_fields(writer.by_ref(), event)?;

//         writeln!(writer)
//     }
// }

// let _subscriber = tracing_subscriber::fmt()
//     .event_format(MyFormatter)
//     .init();

// let _span = tracing::info_span!("my_span", answer = 42).entered();
// tracing::info!(question = "life, the universe, and everything", "hello world");
