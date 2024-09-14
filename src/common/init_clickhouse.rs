use clickhouse::Client;

use crate::config::tuna_config::ClickhouseConfig;

pub async fn init_clickhouse(
    clickhouse_config: &ClickhouseConfig,
) -> Result<Client, anyhow::Error> {
    let client = Client::default()
        .with_url(clickhouse_config.url.clone())
        .with_user(clickhouse_config.user.clone())
        .with_password(clickhouse_config.password.clone())
        .with_database(clickhouse_config.database.clone());
    Ok(client)
}
