use defguard_proxy::{config::get_config, http::run_server, logging::init_tracing, VERSION};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // configuration
    if dotenvy::from_filename(".env.local").is_err() {
        dotenvy::dotenv().ok();
    }

    // read config from env
    let config = get_config()?;
    init_tracing(&config.log_level);
    tracing::info!("Starting ... version v{}", VERSION);

    // run API web server
    run_server(config).await?;

    Ok(())
}
