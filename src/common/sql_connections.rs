use sqlx::Executor;
use sqlx::{mysql::MySqlPool, mysql::MySqlPoolOptions};
use std::env;
use std::time::Duration;
pub async fn create_pool() -> Result<MySqlPool, anyhow::Error> {
    env::set_var("DATABASE_URL", "mysql://root:root@localhost:9306/mydb");
    let database_url = env::var("DATABASE_URL").expect("database_url is not exist");
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
