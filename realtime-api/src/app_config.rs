use anyhow::Result;
use serde::Deserialize;

#[derive(Clone, Deserialize)]
pub struct AppConfig {
    pub api_listener_address: String,
}

impl AppConfig {
    pub fn load(path: &str) -> Result<Self> {
        let text = std::fs::read_to_string(path)?;
        Ok(toml::from_str(&text)?)
    }
}
