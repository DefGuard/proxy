use std::{fs::read_to_string, path::PathBuf};

use clap::Parser;
use log::LevelFilter;
use serde::Deserialize;

#[derive(Parser, Debug, Deserialize)]
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

    #[arg(long, env = "DEFGUARD_PROXY_RATELIMIT_PERSECOND", default_value_t = 0)]
    pub rate_limit_per_second: u64,

    #[arg(long, env = "DEFGUARD_PROXY_RATELIMIT_BURST", default_value_t = 0)]
    pub rate_limit_burst: u32,

    /// Configuration file path
    #[arg(long = "config", short)]
    #[serde(skip)]
    config_path: Option<PathBuf>,
}

#[derive(thiserror::Error, Debug)]
pub enum ConfigError {
    #[error("Failed to read config file")]
    IoError(#[from] std::io::Error),
    #[error("Failed to parse config file")]
    ParseError(#[from] toml::de::Error),
}

pub fn get_config() -> Result<Config, ConfigError> {
    // parse CLI arguments to get config file path
    let cli_config = Config::parse();

    // load config from file if one was specified
    if let Some(config_path) = cli_config.config_path {
        info!("Reading configuration from file: {config_path:?}");
        let config_toml = read_to_string(config_path)?;
        let file_config: Config = toml::from_str(&config_toml)?;
        Ok(file_config)
    } else {
        Ok(cli_config)
    }
}
