use std::{fs::read_to_string, net::IpAddr, path::PathBuf, time::Duration};

use clap::Parser;
use log::LevelFilter;
use serde::Deserialize;

fn default_adoption_timeout() -> u64 {
    10
}

#[derive(Parser, Debug, Deserialize, Clone)]
#[command(version)]
pub struct EnvConfig {
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

    #[arg(long, env = "DEFGUARD_PROXY_LOG_LEVEL", default_value_t = LevelFilter::Info)]
    pub log_level: LevelFilter,

    #[arg(long, env = "DEFGUARD_PROXY_RATELIMIT_PERSECOND", default_value_t = 0)]
    pub rate_limit_per_second: u64,

    #[arg(long, env = "DEFGUARD_PROXY_RATELIMIT_BURST", default_value_t = 0)]
    pub rate_limit_burst: u32,

    /// Configuration file path
    #[arg(long = "config", short)]
    #[serde(skip)]
    config_path: Option<PathBuf>,

    #[arg(long, env = "DEFGUARD_HTTP_BIND_ADDRESS")]
    pub http_bind_address: Option<IpAddr>,

    #[arg(long, env = "DEFGUARD_GRPC_BIND_ADDRESS")]
    pub grpc_bind_address: Option<IpAddr>,

    // TODO: On different platforms this may be different
    #[arg(
        long,
        env = "DEFGUARD_PROXY_CERT_DIR",
        default_value = "/etc/defguard/certs"
    )]
    pub cert_dir: PathBuf,

    /// Port for the HTTPS server. When Core sends TLS certificates over gRPC, the HTTP
    /// server is restarted on this port using those certificates.
    #[arg(long, env = "DEFGUARD_PROXY_HTTPS_PORT", default_value_t = 443)]
    pub https_port: u16,

    /// Use Let's Encrypt staging environment for ACME issuance.
    #[arg(long, env = "DEFGUARD_PROXY_ACME_STAGING", default_value_t = false)]
    pub acme_staging: bool,

    /// Time limit in minutes for the auto-adoption process.
    /// After this time Edge will reject adoption attempts until restarted.
    #[arg(
        long,
        short = 't',
        env = "DEFGUARD_ADOPTION_TIMEOUT",
        default_value = "10"
    )]
    #[serde(default = "default_adoption_timeout")]
    pub adoption_timeout: u64,
}

impl EnvConfig {
    #[must_use]
    pub fn adoption_timeout(&self) -> Duration {
        Duration::from_secs(self.adoption_timeout * 60)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ConfigError {
    #[error("Failed to read config file")]
    IoError(#[from] std::io::Error),
    #[error("Failed to parse config file")]
    ParseError(#[from] toml::de::Error),
}

pub fn get_env_config() -> Result<EnvConfig, ConfigError> {
    // parse CLI arguments to get config file path
    let cli_config = EnvConfig::parse();

    // load config from file if one was specified
    if let Some(config_path) = cli_config.config_path {
        info!("Reading configuration from file: {config_path:?}");
        let config_toml = read_to_string(config_path)?;
        let file_config: EnvConfig = toml::from_str(&config_toml)?;
        Ok(file_config)
    } else {
        Ok(cli_config)
    }
}
