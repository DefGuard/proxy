pub mod assets;
pub mod config;
mod enterprise;
mod error;
mod grpc;
mod handlers;
pub mod http;
pub mod logging;
mod version;

pub(crate) mod proto {
    tonic::include_proto!("defguard.proxy");
}

#[macro_use]
extern crate tracing;

pub static VERSION: &str = concat!(env!("CARGO_PKG_VERSION"), "+", env!("VERGEN_GIT_SHA"));
