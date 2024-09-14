use crate::config::tuna_config::{DatabaseConfig, MysqlConfig};
use sqlx::Executor;
use sqlx::{mysql::MySqlPool, mysql::MySqlPoolOptions};
use std::env;
use std::time::Duration;
pub async fn create_pool(mysql_config: &MysqlConfig) -> Result<MySqlPool, anyhow::Error> {
    let database_url = mysql_config.url.clone();
    let pool = MySqlPoolOptions::new()
        .after_connect(|conn, _| {
            Box::pin(async move {
                let _ = conn.execute("SET time_zone='Asia/Shanghai';").await;
                Ok(())
            })
        })
        .acquire_timeout(Duration::from_secs(5))
        .max_lifetime(Duration::from_secs(3600))
        .idle_timeout(Duration::from_secs(600))
        .connect(&database_url)
        .await?;

    Ok(pool)
}
