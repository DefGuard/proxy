pub mod assets;
pub mod config;
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
