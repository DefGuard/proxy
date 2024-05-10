use clap::Parser;
use tracing::log::LevelFilter;

#[derive(Parser, Debug)]
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

    // port the API server will listen on
    #[arg(long, env = "DEFGUARD_PROXY_GRPC_PORT", default_value_t = 50051)]
    pub grpc_port: u16,

    #[arg(long, env = "DEFGUARD_PROXY_GRPC_CERT")]
    pub grpc_cert: Option<String>,

    #[arg(long, env = "DEFGUARD_PROXY_GRPC_KEY")]
    pub grpc_key: Option<String>,

    #[arg(long, env = "DEFGUARD_PROXY_LOG_LEVEL", default_value_t = LevelFilter::Info)]
    pub log_level: LevelFilter,

    #[arg(
        long,
        env = "DEFGUARD_PROXY_RATELIMIT_PERSECOND",
        default_value_t = 0,
    )]
    pub rate_limit_per_second: u64,

    #[arg(
        long,
        env = "DEFGUARD_PROXY_RATELIMIT_BURST",
        default_value_t = 0,
    )]
    pub rate_limit_burst: u32,
}
