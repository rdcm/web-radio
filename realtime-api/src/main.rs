use anyhow::Result;
use realtime_api::app_config::AppConfig;
use realtime_api::service::Service;

#[tokio::main]
async fn main() -> Result<()> {
    let config = AppConfig::load("config.toml")?;
    let service = Service::new(&config).await?;
    service.run().await
}
