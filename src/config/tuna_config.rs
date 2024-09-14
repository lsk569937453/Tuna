use builder::DefaultState;
use config::Config;
use config::ConfigBuilder;
use config::Environment;
use config::FileFormat;
use config::*;
use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use serde_yaml::from_reader;
use std::env;
use std::path::PathBuf;
#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub clickhouse: DatabaseConfig,
    pub mysql: MysqlConfig,
    pub redis: RedisConfig,
}
// Function providing the default ClickHouse configuration

impl AppConfig {
    pub fn load_config() -> Result<Self, anyhow::Error> {
        dotenv::dotenv().ok();

        let mut builder = ConfigBuilder::<DefaultState>::default();

        builder = builder.add_source(File::new("config/tuna.yaml", FileFormat::Yaml));
        builder = builder.add_source(config::Environment::with_prefix("APP"));

        let config = builder.build()?;
        let app_config: AppConfig = config.try_deserialize()?;

        Ok(app_config)
    }
}
#[derive(Debug, Serialize, Deserialize)]
pub struct MysqlConfig {
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub user: String,
    pub password: String,
    pub database: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RedisConfig {
    pub urls: Vec<String>,
    pub password: String,
}
