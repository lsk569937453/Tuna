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
use std::process::exit;
#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub clickhouse: ClickhouseConfig,
    pub mysql: MysqlConfig,
    pub redis: RedisConfig,
    pub logging: Option<LoggingConfig>,
}
// Function providing the default ClickHouse configuration

impl AppConfig {
    pub fn load_config() -> Self {
        match Self::load_config_with_error() {
            Ok(app_config) => app_config,
            Err(err) => {
                println!("Load error :{}", err);
                exit(1);
            }
        }
    }
    pub fn load_config_with_error() -> Result<Self, anyhow::Error> {
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
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct LoggingConfig {
    pub console: Option<bool>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct ClickhouseConfig {
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
