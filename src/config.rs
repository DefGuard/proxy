use clap::Parser;
use url::Url;

#[derive(Parser)]
#[command(version)]
pub struct Config {
    // port the API server will listen on
    #[arg(
        long,
        short = 'p',
        env = "DEFGUARD_PROXY_HTTP_PORT",
        default_value_t = 8080
    )]
    pub http_port: u16,

    /// Defguard core server gRPC endpoint URL
    #[arg(
        long,
        short = 'g',
        env = "DEFGUARD_PROXY_UPSTREAM_GRPC_URL",
        default_value = "http://localhost:50055/"
    )]
    pub grpc_url: Url,
}
