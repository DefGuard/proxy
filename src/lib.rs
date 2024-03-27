pub mod config;
mod error;
mod grpc;
mod handlers;
pub mod http;
pub mod tracing;

pub(crate) mod proto {
    tonic::include_proto!("defguard.proxy");
}

#[macro_use]
extern crate tracing as rust_tracing;
