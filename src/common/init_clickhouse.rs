use clickhouse::Client;

pub async fn init_clickhouse() -> Result<Client, anyhow::Error> {
    let client = Client::default()
        .with_url("http://localhost:8123")
        .with_user("clickhouse-user")
        .with_password("secret")
        .with_database("tuna");
    Ok(client)
}
