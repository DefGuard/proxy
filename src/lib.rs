use std::sync::Arc;

use defguard_version::Version;
use tokio::sync::mpsc;

use crate::proto::LogEntry;

pub mod assets;
pub mod config;
mod enterprise;
mod error;
pub mod grpc;
mod handlers;
pub mod http;
pub mod logging;
mod setup;

pub(crate) mod proto {
    tonic::include_proto!("defguard.proxy");
}

#[macro_use]
extern crate tracing;

pub static VERSION: &str = concat!(env!("CARGO_PKG_VERSION"), "+", env!("VERGEN_GIT_SHA"));
pub const MIN_CORE_VERSION: Version = Version::new(1, 6, 0);

type CommsChannel<T> = (
    Arc<tokio::sync::Mutex<mpsc::Sender<T>>>,
    Arc<tokio::sync::Mutex<mpsc::Receiver<T>>>,
);

type LogsReceiver = Arc<tokio::sync::Mutex<mpsc::Receiver<LogEntry>>>;
