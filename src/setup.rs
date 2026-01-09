use std::{
    net::SocketAddr,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, LazyLock, Mutex,
    },
};

use defguard_version::{
    server::{grpc::DefguardVersionInterceptor, DefguardVersionLayer},
    DefguardComponent, Version,
};
use tokio::sync::mpsc;
use tonic::{transport::Server, Request, Response, Status};

use crate::{
    error::ApiError,
    grpc::Configuration,
    proto::{proxy_setup_server, DerPayload, InitialSetupInfo},
    CommsChannel, MIN_CORE_VERSION, VERSION,
};

static SETUP_CHANNEL: LazyLock<CommsChannel<Option<Configuration>>> = LazyLock::new(|| {
    let (tx, rx) = mpsc::channel(10);
    (
        Arc::new(tokio::sync::Mutex::new(tx)),
        Arc::new(tokio::sync::Mutex::new(rx)),
    )
});

pub(crate) struct ProxySetupServer {
    setup_in_progress: Arc<AtomicBool>,
    key_pair: Arc<Mutex<Option<defguard_certs::RcGenKeyPair>>>,
}

impl Clone for ProxySetupServer {
    fn clone(&self) -> Self {
        Self {
            setup_in_progress: Arc::clone(&self.setup_in_progress),
            key_pair: Arc::clone(&self.key_pair),
        }
    }
}

impl ProxySetupServer {
    pub fn new() -> Self {
        Self {
            setup_in_progress: Arc::new(AtomicBool::new(false)),
            key_pair: Arc::new(Mutex::new(None)),
        }
    }

    /// Await setup connection from Defguard Core and process it.
    /// Spins up a dedicated temporary gRPC server over HTTP to handle the setup process.
    /// The setup process involves generating a CSR, sending it to Core and receiving signed TLS certificates.
    /// Returns the received gRPC configuration (locally generated key pair and remotely signed certificate) upon successful setup.
    pub(crate) async fn await_setup(
        &self,
        addr: SocketAddr,
    ) -> Result<Configuration, anyhow::Error> {
        info!("gRPC waiting for setup connection from Core on {addr}");
        let server_builder = Server::builder();
        let mut server_config = None;

        let own_version = Version::parse(VERSION)?;

        server_builder
            .layer(tonic::service::InterceptorLayer::new(
                DefguardVersionInterceptor::new(
                    own_version.clone(),
                    DefguardComponent::Core,
                    MIN_CORE_VERSION,
                    false,
                ),
            ))
            .layer(DefguardVersionLayer::new(own_version))
            .add_service(proxy_setup_server::ProxySetupServer::new(self.clone()))
            .serve_with_shutdown(addr, async {
                let config = SETUP_CHANNEL.1.lock().await.recv().await;
                if let Some(cfg) = config {
                    debug!("Received the passed Proxy configuration");
                    server_config = cfg;
                } else {
                    error!("Setup communication channel closed unexpectedly");
                }
            })
            .await
            .map_err(|err| {
                error!("gRPC server error during setup: {err}");
                ApiError::Unexpected("gRPC server error during setup".into())
            })?;

        debug!("gRPC setup server on {addr} has shutdown after completing setup");

        Ok(server_config.map_or_else(
            || {
                error!("No server configuration received after setup completion");
                Err(ApiError::Unexpected(
                    "No server configuration received after setup".into(),
                ))
            },
            |cfg| {
                debug!("Returning received server configuration");
                Ok(cfg)
            },
        )?)
    }
}

#[tonic::async_trait]
impl proxy_setup_server::ProxySetup for ProxySetupServer {
    async fn start(
        &self,
        request: Request<InitialSetupInfo>,
    ) -> Result<Response<DerPayload>, Status> {
        if self.setup_in_progress.load(Ordering::SeqCst) {
            error!("Setup already in progress, rejecting new setup request");
            return Err(Status::resource_exhausted("Setup already in progress"));
        }

        self.setup_in_progress.store(true, Ordering::SeqCst);
        let setup_info = request.into_inner();

        let key_pair = match defguard_certs::generate_key_pair() {
            Ok(kp) => kp,
            Err(err) => {
                error!("Failed to generate key pair: {err}");
                self.setup_in_progress.store(false, Ordering::SeqCst);
                return Err(Status::internal("Failed to generate key pair"));
            }
        };

        let subject_alt_names = vec![setup_info.cert_hostname];

        let csr = match defguard_certs::Csr::new(
            &key_pair,
            &subject_alt_names,
            vec![
                // TODO: Change it?
                (defguard_certs::DnType::CommonName, "Defguard Proxy"),
                (defguard_certs::DnType::OrganizationName, "Defguard"),
            ],
        ) {
            Ok(csr) => csr,
            Err(err) => {
                error!("Failed to generate CSR: {err}");
                self.setup_in_progress.store(false, Ordering::SeqCst);
                return Err(Status::internal(format!("Failed to generate CSR: {err}")));
            }
        };

        self.key_pair.lock().unwrap().replace(key_pair);

        let csr_der = csr.to_der();
        let csr_request = DerPayload {
            der_data: csr_der.to_vec(),
        };

        Ok(Response::new(csr_request))
    }

    async fn send_cert(&self, request: Request<DerPayload>) -> Result<Response<()>, Status> {
        let der_payload = request.into_inner();
        let cert_der = der_payload.der_data;

        let grpc_cert_pem =
            match defguard_certs::der_to_pem(&cert_der, defguard_certs::PemLabel::Certificate) {
                Ok(pem) => pem,
                Err(err) => {
                    error!("Failed to convert certificate DER to PEM: {err}");
                    self.setup_in_progress.store(false, Ordering::SeqCst);
                    return Err(Status::internal(format!(
                        "Failed to convert certificate DER to PEM: {err}"
                    )));
                }
            };

        let key_pair = {
            let key_pair = self.key_pair.lock().unwrap().take();
            if let Some(kp) = key_pair {
                kp
            } else {
                error!("Key pair not found during Proxy setup. Key pair generation step might have failed.");
                self.setup_in_progress.store(false, Ordering::SeqCst);
                return Err(Status::internal("Key pair not found during Proxy setup. Key pair generation step might have failed."));
            }
        };

        let configuration = Configuration {
            grpc_key_pem: key_pair.serialize_pem(),
            grpc_cert_pem,
        };

        match SETUP_CHANNEL.0.lock().await.send(Some(configuration)).await {
            Ok(()) => info!("Configuration sent to gRPC server"),
            Err(err) => {
                error!("Failed to send configuration to gRPC server: {err}");
                self.setup_in_progress.store(false, Ordering::SeqCst);
                return Err(Status::internal(
                    "Failed to send configuration to gRPC server",
                ));
            }
        }

        self.setup_in_progress.store(false, Ordering::SeqCst);

        Ok(Response::new(()))
    }
}
