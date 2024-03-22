use std::fmt;
use rust_tracing::{Subscriber, Event};
// use tracing_core::{Subscriber, Event};
use tracing_subscriber::fmt::{
    format::{self, FormatEvent, FormatFields, Full, Writer},
    FmtContext,
    FormattedFields, time::{SystemTime, FormatTime},
};
use tracing_subscriber::registry::LookupSpan;

// pub(crate) struct HttpFormatter;

#[derive(Debug, Clone)]
pub struct HttpFormatter<F = Full, T = SystemTime> {
    format: F,
    pub(crate) timer: T,
    pub(crate) ansi: Option<bool>,
    pub(crate) display_timestamp: bool,
    pub(crate) display_target: bool,
    pub(crate) display_level: bool,
    pub(crate) display_thread_id: bool,
    pub(crate) display_thread_name: bool,
    pub(crate) display_filename: bool,
    pub(crate) display_line_number: bool,
}

impl<S, N, T> FormatEvent<S, N> for HttpFormatter<Full, T>
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
    T: FormatTime,
{
    fn format_event(
        &self,
        ctx: &FmtContext<'_, S, N>,
        mut writer: Writer<'_>,
        event: &Event<'_>,
    ) -> fmt::Result {
        #[cfg(feature = "tracing-log")]
        let normalized_meta = event.normalized_metadata();
        #[cfg(feature = "tracing-log")]
        let meta = normalized_meta.as_ref().unwrap_or_else(|| event.metadata());
        #[cfg(not(feature = "tracing-log"))]
        let meta = event.metadata();

        // if the `Format` struct *also* has an ANSI color configuration,
        // override the writer...the API for configuring ANSI color codes on the
        // `Format` struct is deprecated, but we still need to honor those
        // configurations.
        // if let Some(ansi) = self.ansi {
        //     writer = writer.with_ansi(ansi);
        // }

        self.format_timestamp(&mut writer)?;

        if self.display_level {
            let fmt_level = {
                #[cfg(feature = "ansi")]
                {
                    FmtLevel::new(meta.level(), writer.has_ansi_escapes())
                }
                #[cfg(not(feature = "ansi"))]
                {
                    FmtLevel::new(meta.level())
                }
            };
            write!(writer, "{} ", fmt_level)?;
        }

        if self.display_thread_name {
            let current_thread = std::thread::current();
            match current_thread.name() {
                Some(name) => {
                    write!(writer, "{} ", FmtThreadName::new(name))?;
                }
                // fall-back to thread id when name is absent and ids are not enabled
                None if !self.display_thread_id => {
                    write!(writer, "{:0>2?} ", current_thread.id())?;
                }
                _ => {}
            }
        }

        if self.display_thread_id {
            write!(writer, "{:0>2?} ", std::thread::current().id())?;
        }

        let dimmed = writer.dimmed();

        if let Some(scope) = ctx.event_scope() {
            let bold = writer.bold();

            let mut seen = false;

            for span in scope.from_root() {
                write!(writer, "{}", bold.paint(span.metadata().name()))?;
                seen = true;

                let ext = span.extensions();
                if let Some(fields) = &ext.get::<FormattedFields<N>>() {
                    if !fields.is_empty() {
                        write!(writer, "{}{}{}", bold.paint("{"), fields, bold.paint("}"))?;
                    }
                }
                write!(writer, "{}", dimmed.paint(":"))?;
            }

            if seen {
                writer.write_char(' ')?;
            }
        };

        if self.display_target {
            write!(
                writer,
                "{}{} ",
                dimmed.paint(meta.target()),
                dimmed.paint(":")
            )?;
        }

        let line_number = if self.display_line_number {
            meta.line()
        } else {
            None
        };

        if self.display_filename {
            if let Some(filename) = meta.file() {
                write!(
                    writer,
                    "{}{}{}",
                    dimmed.paint(filename),
                    dimmed.paint(":"),
                    if line_number.is_some() { "" } else { " " }
                )?;
            }
        }

        if let Some(line_number) = line_number {
            write!(
                writer,
                "{}{}:{} ",
                dimmed.prefix(),
                line_number,
                dimmed.suffix()
            )?;
        }

        ctx.format_fields(writer.by_ref(), event)?;
        writeln!(writer)
    }
}

impl<F, T> HttpFormatter<F, T> {
    /// Use a less verbose output format.
    ///
    ///// See [`Compact`].
    //pub fn compact(self) -> HttpFormatter<Compact, T> {
    //    Format {
    //        format: Compact,
    //        timer: self.timer,
    //        ansi: self.ansi,
    //        display_target: self.display_target,
    //        display_timestamp: self.display_timestamp,
    //        display_level: self.display_level,
    //        display_thread_id: self.display_thread_id,
    //        display_thread_name: self.display_thread_name,
    //        display_filename: self.display_filename,
    //        display_line_number: self.display_line_number,
    //    }
    //}

    ///// Use an excessively pretty, human-readable output format.
    /////
    ///// See [`Pretty`].
    /////
    ///// Note that this requires the "ansi" feature to be enabled.
    /////
    ///// # Options
    /////
    ///// [`Format::with_ansi`] can be used to disable ANSI terminal escape codes (which enable
    ///// formatting such as colors, bold, italic, etc) in event formatting. However, a field
    ///// formatter must be manually provided to avoid ANSI in the formatting of parent spans, like
    ///// so:
    /////
    ///// ```
    ///// # use tracing_subscriber::fmt::format;
    ///// tracing_subscriber::fmt()
    /////    .pretty()
    /////    .with_ansi(false)
    /////    .fmt_fields(format::PrettyFields::new().with_ansi(false))
    /////    // ... other settings ...
    /////    .init();
    ///// ```
    //#[cfg(feature = "ansi")]
    //#[cfg_attr(docsrs, doc(cfg(feature = "ansi")))]
    //pub fn pretty(self) -> Format<Pretty, T> {
    //    Format {
    //        format: Pretty::default(),
    //        timer: self.timer,
    //        ansi: self.ansi,
    //        display_target: self.display_target,
    //        display_timestamp: self.display_timestamp,
    //        display_level: self.display_level,
    //        display_thread_id: self.display_thread_id,
    //        display_thread_name: self.display_thread_name,
    //        display_filename: true,
    //        display_line_number: true,
    //    }
    //}

    ///// Use the full JSON format.
    /////
    ///// The full format includes fields from all entered spans.
    /////
    ///// # Example Output
    /////
    ///// ```ignore,json
    ///// {"timestamp":"Feb 20 11:28:15.096","level":"INFO","target":"mycrate","fields":{"message":"some message", "key": "value"}}
    ///// ```
    /////
    ///// # Options
    /////
    ///// - [`Format::flatten_event`] can be used to enable flattening event fields into the root
    ///// object.
    //#[cfg(feature = "json")]
    //#[cfg_attr(docsrs, doc(cfg(feature = "json")))]
    //pub fn json(self) -> Format<Json, T> {
    //    Format {
    //        format: Json::default(),
    //        timer: self.timer,
    //        ansi: self.ansi,
    //        display_target: self.display_target,
    //        display_timestamp: self.display_timestamp,
    //        display_level: self.display_level,
    //        display_thread_id: self.display_thread_id,
    //        display_thread_name: self.display_thread_name,
    //        display_filename: self.display_filename,
    //        display_line_number: self.display_line_number,
    //    }
    //}

    ///// Use the given [`timer`] for log message timestamps.
    /////
    ///// See [`time` module] for the provided timer implementations.
    /////
    ///// Note that using the `"time"` feature flag enables the
    ///// additional time formatters [`UtcTime`] and [`LocalTime`], which use the
    ///// [`time` crate] to provide more sophisticated timestamp formatting
    ///// options.
    /////
    ///// [`timer`]: super::time::FormatTime
    ///// [`time` module]: mod@super::time
    ///// [`UtcTime`]: super::time::UtcTime
    ///// [`LocalTime`]: super::time::LocalTime
    ///// [`time` crate]: https://docs.rs/time/0.3
    //pub fn with_timer<T2>(self, timer: T2) -> Format<F, T2> {
    //    Format {
    //        format: self.format,
    //        timer,
    //        ansi: self.ansi,
    //        display_target: self.display_target,
    //        display_timestamp: self.display_timestamp,
    //        display_level: self.display_level,
    //        display_thread_id: self.display_thread_id,
    //        display_thread_name: self.display_thread_name,
    //        display_filename: self.display_filename,
    //        display_line_number: self.display_line_number,
    //    }
    //}

    ///// Do not emit timestamps with log messages.
    //pub fn without_time(self) -> Format<F, ()> {
    //    Format {
    //        format: self.format,
    //        timer: (),
    //        ansi: self.ansi,
    //        display_timestamp: false,
    //        display_target: self.display_target,
    //        display_level: self.display_level,
    //        display_thread_id: self.display_thread_id,
    //        display_thread_name: self.display_thread_name,
    //        display_filename: self.display_filename,
    //        display_line_number: self.display_line_number,
    //    }
    //}

    ///// Enable ANSI terminal colors for formatted output.
    //pub fn with_ansi(self, ansi: bool) -> Format<F, T> {
    //    Format {
    //        ansi: Some(ansi),
    //        ..self
    //    }
    //}

    ///// Sets whether or not an event's target is displayed.
    //pub fn with_target(self, display_target: bool) -> Format<F, T> {
    //    Format {
    //        display_target,
    //        ..self
    //    }
    //}

    ///// Sets whether or not an event's level is displayed.
    //pub fn with_level(self, display_level: bool) -> Format<F, T> {
    //    Format {
    //        display_level,
    //        ..self
    //    }
    //}

    ///// Sets whether or not the [thread ID] of the current thread is displayed
    ///// when formatting events.
    /////
    ///// [thread ID]: std::thread::ThreadId
    //pub fn with_thread_ids(self, display_thread_id: bool) -> Format<F, T> {
    //    Format {
    //        display_thread_id,
    //        ..self
    //    }
    //}

    ///// Sets whether or not the [name] of the current thread is displayed
    ///// when formatting events.
    /////
    ///// [name]: std::thread#naming-threads
    //pub fn with_thread_names(self, display_thread_name: bool) -> Format<F, T> {
    //    Format {
    //        display_thread_name,
    //        ..self
    //    }
    //}

    ///// Sets whether or not an event's [source code file path][file] is
    ///// displayed.
    /////
    ///// [file]: tracing_core::Metadata::file
    //pub fn with_file(self, display_filename: bool) -> Format<F, T> {
    //    Format {
    //        display_filename,
    //        ..self
    //    }
    //}

    ///// Sets whether or not an event's [source code line number][line] is
    ///// displayed.
    /////
    ///// [line]: tracing_core::Metadata::line
    //pub fn with_line_number(self, display_line_number: bool) -> Format<F, T> {
    //    Format {
    //        display_line_number,
    //        ..self
    //    }
    //}

    ///// Sets whether or not the source code location from which an event
    ///// originated is displayed.
    /////
    ///// This is equivalent to calling [`Format::with_file`] and
    ///// [`Format::with_line_number`] with the same value.
    //pub fn with_source_location(self, display_location: bool) -> Self {
    //    self.with_line_number(display_location)
    //        .with_file(display_location)
    //}

    #[inline]
    fn format_timestamp(&self, writer: &mut Writer<'_>) -> fmt::Result
    where
        T: FormatTime,
    {
        // If timestamps are disabled, do nothing.
        if !self.display_timestamp {
            return Ok(());
        }

        // If ANSI color codes are enabled, format the timestamp with ANSI
        // colors.
        #[cfg(feature = "ansi")]
        {
            if writer.has_ansi_escapes() {
                let style = Style::new().dimmed();
                write!(writer, "{}", style.prefix())?;

                // If getting the timestamp failed, don't bail --- only bail on
                // formatting errors.
                if self.timer.format_time(writer).is_err() {
                    writer.write_str("<unknown time>")?;
                }

                write!(writer, "{} ", style.suffix())?;
                return Ok(());
            }
        }

        // Otherwise, just format the timestamp without ANSI formatting.
        // If getting the timestamp failed, don't bail --- only bail on
        // formatting errors.
        if self.timer.format_time(writer).is_err() {
            writer.write_str("<unknown time>")?;
        }
        writer.write_char(' ')
    }
}

// impl<S, N> FormatEvent<S, N> for HttpFormatter
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
//                 write!(writer, "FIELDS: {}", format!("{fields}"))?;
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

impl Default for HttpFormatter<Full, SystemTime> {
    fn default() -> Self {
        HttpFormatter {
            format: Full,
            timer: SystemTime,
            ansi: None,
            display_timestamp: true,
            display_target: true,
            display_level: true,
            display_thread_id: false,
            display_thread_name: false,
            display_filename: false,
            display_line_number: false,
        }
    }
}
// let _subscriber = tracing_subscriber::fmt()
//     .event_format(MyFormatter)
//     .init();

// let _span = tracing::info_span!("my_span", answer = 42).entered();
// tracing::info!(question = "life, the universe, and everything", "hello world");
