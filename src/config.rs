use std::{fs::read_to_string, net::IpAddr, path::PathBuf, time::Duration};

use clap::Parser;
use log::LevelFilter;
use serde::Deserialize;

fn default_http_port() -> u16 {
    8080
}

fn default_grpc_port() -> u16 {
    50051
}

fn default_log_level() -> LevelFilter {
    LevelFilter::Info
}

fn default_cert_dir() -> PathBuf {
    PathBuf::from("/etc/defguard/certs")
}

fn default_https_port() -> u16 {
    443
}

fn default_adoption_timeout() -> u64 {
    10
}

pub(crate) const DEFAULT_GRPC_KEEPALIVE_INTERVAL_SECONDS: u64 = 10;
pub(crate) const DEFAULT_GRPC_KEEPALIVE_TIMEOUT_SECONDS: u64 = 20;

fn default_grpc_keepalive_interval_seconds() -> u64 {
    DEFAULT_GRPC_KEEPALIVE_INTERVAL_SECONDS
}

fn default_grpc_keepalive_timeout_seconds() -> u64 {
    DEFAULT_GRPC_KEEPALIVE_TIMEOUT_SECONDS
}

fn default_rate_limit_per_second() -> u64 {
    0
}

fn default_rate_limit_burst() -> u32 {
    0
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
    #[serde(default = "default_http_port")]
    pub http_port: u16,

    // port the API server will listen on
    #[arg(long, env = "DEFGUARD_PROXY_GRPC_PORT", default_value_t = 50051)]
    #[serde(default = "default_grpc_port")]
    pub grpc_port: u16,

    /// HTTP/2 keepalive ping interval for the gRPC server, in seconds.
    #[arg(
        long,
        env = "DEFGUARD_PROXY_GRPC_KEEPALIVE_INTERVAL_SECONDS",
        default_value_t = DEFAULT_GRPC_KEEPALIVE_INTERVAL_SECONDS
    )]
    #[serde(default = "default_grpc_keepalive_interval_seconds")]
    pub grpc_keepalive_interval_seconds: u64,

    /// Time to wait for an HTTP/2 keepalive ping acknowledgement, in seconds.
    #[arg(
        long,
        env = "DEFGUARD_PROXY_GRPC_KEEPALIVE_TIMEOUT_SECONDS",
        default_value_t = DEFAULT_GRPC_KEEPALIVE_TIMEOUT_SECONDS
    )]
    #[serde(default = "default_grpc_keepalive_timeout_seconds")]
    pub grpc_keepalive_timeout_seconds: u64,

    #[arg(long, env = "DEFGUARD_PROXY_LOG_LEVEL", default_value_t = LevelFilter::Info)]
    #[serde(default = "default_log_level")]
    pub log_level: LevelFilter,

    #[arg(long, env = "DEFGUARD_PROXY_RATELIMIT_PERSECOND", default_value_t = 0)]
    #[serde(default = "default_rate_limit_per_second")]
    pub rate_limit_per_second: u64,

    #[arg(long, env = "DEFGUARD_PROXY_RATELIMIT_BURST", default_value_t = 0)]
    #[serde(default = "default_rate_limit_burst")]
    pub rate_limit_burst: u32,

    /// Configuration file path
    #[arg(long = "config", short)]
    #[serde(skip)]
    config_path: Option<PathBuf>,

    #[arg(long, env = "DEFGUARD_HTTP_BIND_ADDRESS")]
    #[serde(default)]
    pub http_bind_address: Option<IpAddr>,

    #[arg(long, env = "DEFGUARD_GRPC_BIND_ADDRESS")]
    #[serde(default)]
    pub grpc_bind_address: Option<IpAddr>,

    // TODO: On different platforms this may be different
    #[arg(
        long,
        env = "DEFGUARD_PROXY_CERT_DIR",
        default_value = "/etc/defguard/certs"
    )]
    #[serde(default = "default_cert_dir")]
    pub cert_dir: PathBuf,

    /// Port for the HTTPS server. When Core sends TLS certificates over gRPC, the HTTP
    /// server is restarted on this port using those certificates.
    #[arg(long, env = "DEFGUARD_PROXY_HTTPS_PORT", default_value_t = 443)]
    #[serde(default = "default_https_port")]
    pub https_port: u16,

    /// Use Let's Encrypt staging environment for ACME issuance.
    #[arg(long, env = "DEFGUARD_PROXY_ACME_STAGING", default_value_t = false)]
    #[serde(default)]
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
    #[error("gRPC keepalive interval and timeout must be greater than zero seconds")]
    InvalidGrpcKeepalive,
}

impl EnvConfig {
    fn validate(&self) -> Result<(), ConfigError> {
        if self.grpc_keepalive_interval_seconds == 0 || self.grpc_keepalive_timeout_seconds == 0 {
            return Err(ConfigError::InvalidGrpcKeepalive);
        }
        Ok(())
    }
}

pub fn get_env_config() -> Result<EnvConfig, ConfigError> {
    let cli_config = EnvConfig::parse();

    let config = if let Some(config_path) = cli_config.config_path {
        info!("Reading configuration from file: {config_path:?}");
        let config_toml = read_to_string(config_path)?;
        toml::from_str(&config_toml)?
    } else {
        cli_config
    };

    config.validate()?;
    Ok(config)
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use super::{
        ConfigError, DEFAULT_GRPC_KEEPALIVE_INTERVAL_SECONDS,
        DEFAULT_GRPC_KEEPALIVE_TIMEOUT_SECONDS, EnvConfig,
    };

    #[test]
    fn keepalive_defaults_and_toml_values_parse() {
        let defaults: EnvConfig = toml::from_str("").expect("empty TOML should use defaults");
        assert_eq!(
            defaults.grpc_keepalive_interval_seconds,
            DEFAULT_GRPC_KEEPALIVE_INTERVAL_SECONDS
        );
        assert_eq!(
            defaults.grpc_keepalive_timeout_seconds,
            DEFAULT_GRPC_KEEPALIVE_TIMEOUT_SECONDS
        );

        let configured: EnvConfig = toml::from_str(
            "grpc_keepalive_interval_seconds = 15\ngrpc_keepalive_timeout_seconds = 30",
        )
        .expect("keepalive TOML values should parse");
        assert_eq!(configured.grpc_keepalive_interval_seconds, 15);
        assert_eq!(configured.grpc_keepalive_timeout_seconds, 30);
    }

    #[test]
    fn keepalive_cli_options_parse_seconds() {
        let config = EnvConfig::try_parse_from([
            "edge",
            "--grpc-keepalive-interval-seconds",
            "15",
            "--grpc-keepalive-timeout-seconds",
            "30",
        ])
        .expect("keepalive CLI options should parse");
        assert_eq!(config.grpc_keepalive_interval_seconds, 15);
        assert_eq!(config.grpc_keepalive_timeout_seconds, 30);
    }

    #[test]
    fn keepalive_environment_variables_are_loaded_in_child() {
        let output = std::process::Command::new(
            std::env::current_exe().expect("test executable path should exist"),
        )
        .args([
            "--exact",
            "config::tests::keepalive_environment_variables_child",
            "--nocapture",
        ])
        .env("DEFGUARD_TEST_KEEPALIVE_ENV_CHILD", "1")
        .env("DEFGUARD_PROXY_GRPC_KEEPALIVE_INTERVAL_SECONDS", "13")
        .env("DEFGUARD_PROXY_GRPC_KEEPALIVE_TIMEOUT_SECONDS", "27")
        .output()
        .expect("environment test child should start");
        assert!(
            output.status.success(),
            "child test failed:\n{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    #[test]
    fn keepalive_environment_variables_child() {
        if std::env::var_os("DEFGUARD_TEST_KEEPALIVE_ENV_CHILD").is_none() {
            return;
        }

        let config = EnvConfig::try_parse_from(["edge"]).expect("environment values should parse");
        assert_eq!(config.grpc_keepalive_interval_seconds, 13);
        assert_eq!(config.grpc_keepalive_timeout_seconds, 27);
    }

    #[test]
    fn keepalive_values_must_be_positive() {
        for toml in [
            "grpc_keepalive_interval_seconds = 0",
            "grpc_keepalive_timeout_seconds = 0",
        ] {
            let config: EnvConfig = toml::from_str(toml).expect("zero parses before validation");
            assert!(matches!(
                config.validate(),
                Err(ConfigError::InvalidGrpcKeepalive)
            ));
        }
    }
}
