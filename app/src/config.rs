use config::{Config as ConfigBuilder, File};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub database_url: String,
    pub tmdb_api_key: String,
    pub port: u16,
}

impl Config {
    pub fn new() -> anyhow::Result<Self> {
        dotenvy::dotenv().ok();

        let config = ConfigBuilder::builder()
            .add_source(File::with_name("config").required(false))
            .set_default("database_url", "sqlite://./streaming.db")?
            .set_default("port", 3000u16)?
            .build()?;

        let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            config
                .get_string("database_url")
                .unwrap_or_else(|_| "sqlite://./streaming.db".to_string())
        });

        Ok(Config {
            database_url,
            tmdb_api_key: std::env::var("TMDB_API_KEY")
                .map_err(|_| anyhow::anyhow!("TMDB_API_KEY environment variable not set"))?,
            port: std::env::var("PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or_else(|| config.get_int("port").unwrap_or(3000) as u16),
        })
    }
}
