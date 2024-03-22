use tracing::log::LevelFilter;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use tracing::{Event, Subscriber};
use tracing_subscriber::{layer::Context, registry::LookupSpan, Layer};

use crate::http_tracing_formatter::HttpFormatter;

// struct HttpLayer<S> {
//     inner: fmt::Layer<S>,
// }

// impl<S> HttpLayer<S> {
//     pub fn new() -> Self {
//         HttpLayer {
//             inner: fmt::layer(),
//         }
//     }
// }

// impl<S: Subscriber> Layer<S> for HttpLayer<S>
// where
//     S: for<'a> LookupSpan<'a>,
// {
//     fn on_event(&self, event: &Event<'_>, ctx: Context<'_, S>) {
//         // let mut visitor = fmt::format::DefaultFields::default();
//         // event.record(&mut visitor);
//         // let fields = visitor.fields.as_ref().map_or("", String::as_str);

//         if let Some(span) = ctx.lookup_current() {
//             // if span.name() == "http_request" {
//             if span.name() == "start_enrollment_process" {
//                 let mut client_addr = "".to_string();
//                 let mut method = "".to_string();
//                 let mut path = "".to_string();

//                 span.extensions()
//                     .get::<std::sync::Mutex<HttpSpanData>>()
//                     .map(|data| {
//                         let data = data.lock().unwrap();
//                         client_addr = data.client_addr.clone();
//                         method = data.method.clone();
//                         path = data.path.clone();
//                     });

//                 // println!("{} {} {}. {}", client_addr, method, path, fields);
//                 println!("{} {} {}", client_addr, method, path);
//                 return;
//             }
//         }

//         self.inner.on_event(event, ctx);
//     }
// }

// // Define your HttpSpanData struct to store span-specific data
// struct HttpSpanData {
//     client_addr: String,
//     method: String,
//     path: String,
// }

// // Initialize and use your custom layer
// let custom_layer = HttpLayer::new().with_subscriber(tracing_subscriber::registry().with(
//     EnvFilter::try_from_env("DEFGUARD_PROXY_LOG_FILTER")
//         .unwrap_or_else(|_| level.to_string().into()),
// ));

// tracing::subscriber::set_global_default(custom_layer)
// .expect("Failed to set subscriber");

// Initializes tracing with the specified log level.
// Allows fine-grained filtering with `EnvFilter` directives.
// The directives are read from `DEFGUARD_PROXY_LOG_FILTER` env variable.
// For more info check: <https://docs.rs/tracing-subscriber/latest/tracing_subscriber/filter/struct.EnvFilter.html>
pub fn init_tracing(level: &LevelFilter) {

    // let _subscriber = tracing_subscriber::fmt()
    //     .event_format(HttpFormatter)
    //     .init();
    tracing_subscriber::registry()
        .with(
            EnvFilter::try_from_env("DEFGUARD_PROXY_LOG_FILTER")
                .unwrap_or_else(|_| level.to_string().into()),
        )
        // .with(fmt::layer().event_format(HttpFormatter))
        .with(fmt::layer())
        // .with(HttpLayer::new())
        .init();
    info!("Tracing initialized");
}
