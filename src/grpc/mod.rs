pub(crate) mod enrollment;
pub(crate) mod password_reset;

use std::{fs::read_to_string, time::Duration};

use crate::config::Config;
use thiserror::Error;
use tonic::transport::{Certificate, Channel, ClientTlsConfig, Endpoint};
use tracing::debug;

pub const TEN_SECS: Duration = Duration::from_secs(10);

#[derive(Debug, Error)]
pub(crate) enum GrpcError {
    #[error("gRPC client setup error: {0}")]
    ClientSetup(String),
    #[error(transparent)]
    TransportError(#[from] tonic::transport::Error),
}

pub(crate) fn setup_channel(config: &Config) -> Result<Channel, GrpcError> {
    debug!("Setting up gRPC channel");
    let endpoint = Endpoint::from_shared(config.grpc_url.to_string())?;
    let endpoint = endpoint.http2_keep_alive_interval(TEN_SECS);
    let endpoint = endpoint.tcp_keepalive(Some(TEN_SECS));
    let endpoint = if let Some(ca) = config.grpc_ca.clone() {
        let ca = read_to_string(ca).map_err(|err| GrpcError::ClientSetup(err.to_string()))?;
        let tls = ClientTlsConfig::new().ca_certificate(Certificate::from_pem(ca));
        endpoint.tls_config(tls)?
    } else {
        endpoint
    };

    Ok(endpoint.connect_lazy())
}
