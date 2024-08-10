use sqlx::MySql;
use sqlx::Pool;

pub async fn init_with_error(pool: Pool<MySql>) -> Result<(), anyhow::Error> {
    migrate(pool.clone()).await?;
    Ok(())
}
async fn migrate(pool: Pool<MySql>) -> Result<(), anyhow::Error> {
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .map_err(|e| anyhow!("{}", e))
}
