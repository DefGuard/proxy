//! Connection settings for the public HTTP(S) listener.
//!
//! Without these, a client that disappears without closing its connection (sleep, network
//! change, NAT timeout) keeps a file descriptor open forever, and the Edge eventually stops
//! accepting connections once it reaches the open file limit.

use std::time::Duration;

use axum_server::{
    Address, Server,
    accept::{TcpConfigAcceptor, TcpKeepaliveConfig, TcpSocketConfig},
};

const HTTP1_HEADER_READ_TIMEOUT: Duration = Duration::from_secs(30);
const HTTP2_KEEPALIVE_INTERVAL: Duration = Duration::from_secs(20);
const HTTP2_KEEPALIVE_TIMEOUT: Duration = Duration::from_secs(20);
const TCP_KEEPALIVE_TIME: Duration = Duration::from_secs(60);
const TCP_KEEPALIVE_INTERVAL: Duration = Duration::from_secs(15);

#[derive(Clone, Copy, Debug)]
pub(crate) struct ConnectionTimeouts {
    /// Also bounds how long an idle HTTP/1 keep-alive connection waits for its next request.
    pub(crate) http1_header_read: Duration,
    pub(crate) http2_keepalive_interval: Duration,
    pub(crate) http2_keepalive_timeout: Duration,
}

impl Default for ConnectionTimeouts {
    fn default() -> Self {
        Self {
            http1_header_read: HTTP1_HEADER_READ_TIMEOUT,
            http2_keepalive_interval: HTTP2_KEEPALIVE_INTERVAL,
            http2_keepalive_timeout: HTTP2_KEEPALIVE_TIMEOUT,
        }
    }
}

/// axum-server installs a timer but turns hyper's header-read timeout off and sends no
/// HTTP/2 keepalive pings, so both have to be set explicitly.
pub(crate) fn configure_timeouts<A: Address, Acc>(
    server: &mut Server<A, Acc>,
    timeouts: ConnectionTimeouts,
) {
    let builder = server.http_builder();
    builder
        .http1()
        .header_read_timeout(timeouts.http1_header_read);
    builder
        .http2()
        .keep_alive_interval(timeouts.http2_keepalive_interval)
        .keep_alive_timeout(timeouts.http2_keepalive_timeout);
}

/// TCP keepalive detects dead peers on connections that hyper's timers no longer cover, such
/// as upgraded WebSocket connections.
pub(crate) fn keepalive_acceptor() -> TcpConfigAcceptor {
    TcpConfigAcceptor::new(
        TcpSocketConfig::new().keepalive(
            TcpKeepaliveConfig::new()
                .time(TCP_KEEPALIVE_TIME)
                .interval(TCP_KEEPALIVE_INTERVAL),
        ),
    )
}
