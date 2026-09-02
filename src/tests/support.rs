//! Shared fixtures for the proxy's unit and handler tests.

use std::{
    path::PathBuf,
    sync::{Arc, RwLock},
};

use axum_extra::extract::cookie::Key;
use tokio::sync::{Mutex, broadcast, mpsc};

use crate::{grpc::ProxyServer, proto::PublicSettings};

/// A `ProxyServer` with throwaway channels, suitable for driving handlers in tests.
pub(crate) fn test_proxy_server(cookie_key: Arc<RwLock<Option<Key>>>) -> ProxyServer {
    let (reset_tx, _) = broadcast::channel(1);
    let (https_cert_tx, _) = broadcast::channel(1);
    let (clear_https_tx, _) = broadcast::channel(1);
    let (_, logs_rx) = mpsc::channel(1);
    ProxyServer::new(
        cookie_key,
        PathBuf::new(),
        reset_tx,
        https_cert_tx,
        clear_https_tx,
        None,
        Arc::new(Mutex::new(logs_rx)),
        false,
    )
}

/// `PublicSettings` with both display flags on, varying only the public URL under test.
pub(crate) fn test_public_settings(public_url: Option<&str>) -> PublicSettings {
    PublicSettings {
        display_password_reset: true,
        display_download_step: true,
        public_url: public_url.map(str::to_owned),
    }
}
