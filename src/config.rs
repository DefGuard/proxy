use clap::Parser;

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

    // port the API server will listen on
    #[arg(long, env = "DEFGUARD_PROXY_GRPC_PORT", default_value_t = 50051)]
    pub grpc_port: u16,
}
