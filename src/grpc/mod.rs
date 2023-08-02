pub mod enrollment;

use crate::config::Config;
use enrollment::proto::enrollment_service_client::EnrollmentServiceClient;
use std::time::Duration;
use thiserror::Error;
use tonic::transport::{Certificate, Channel, ClientTlsConfig, Endpoint};
use tracing::debug;

#[derive(Debug, Error)]
pub enum GrpcError {
    #[error("gRPC client setup error: {0}")]
    ClientSetup(String),
    #[error(transparent)]
    TransportError(#[from] tonic::transport::Error),
}

pub fn setup_client(config: &Config) -> Result<EnrollmentServiceClient<Channel>, GrpcError> {
    debug!("Setting up gRPC client");
    let endpoint = Endpoint::from_shared(config.grpc_url.to_string())?;
    let endpoint = endpoint.http2_keep_alive_interval(Duration::from_secs(10));
    let endpoint = endpoint.tcp_keepalive(Some(Duration::from_secs(10)));
    let endpoint = if let Some(ca) = config.grpc_ca.clone() {
        let ca =
            std::fs::read_to_string(ca).map_err(|err| GrpcError::ClientSetup(err.to_string()))?;
        let tls = ClientTlsConfig::new().ca_certificate(Certificate::from_pem(ca));
        endpoint.tls_config(tls)?
    } else {
        endpoint
    };
    let channel = endpoint.connect_lazy();

    let client = EnrollmentServiceClient::new(channel);

    Ok(client)
}
