use sqlx::{mysql::MySqlConnectOptions, mysql::MySqlPool};
use std::env;
use std::str::FromStr;
pub async fn create_pool() -> Result<MySqlPool, anyhow::Error> {
    env::set_var("DATABASE_URL", "mysql://root:root@localhost:9306/mydb");
    let database_url = env::var("DATABASE_URL").expect("database_url is not exist");
    let options = MySqlConnectOptions::from_str(&database_url)?;

    let pool = MySqlPool::connect_with(options).await?;
    Ok(pool)
}
