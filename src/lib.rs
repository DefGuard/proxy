use std::sync::Arc;

use defguard_version::Version;
use tokio::sync::mpsc;

pub mod assets;
pub mod config;
mod enterprise;
mod error;
mod grpc;
mod handlers;
pub mod http;
pub mod logging;

pub(crate) mod proto {
    tonic::include_proto!("defguard.proxy");
}

#[macro_use]
extern crate tracing;

pub static VERSION: &str = concat!(env!("CARGO_PKG_VERSION"), "+", env!("VERGEN_GIT_SHA"));
pub const MIN_CORE_VERSION: Version = Version::new(1, 5, 0);

type CommsChannel<T> = (
    Arc<tokio::sync::Mutex<mpsc::Sender<T>>>,
    Arc<tokio::sync::Mutex<mpsc::Receiver<T>>>,
);
