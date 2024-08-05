use crate::dao::user::SUPER_ADMIN_AUTHORITY;
use sqlx::mysql::MySqlRow;
use sqlx::MySql;
use sqlx::Pool;
use sqlx::Row;
use tracing::info;
use uuid::Uuid;

pub async fn init_with_error(pool: Pool<MySql>) -> Result<(), anyhow::Error> {
    migrate(pool.clone()).await?;
    // init_super_user(pool).await?;
    Ok(())
}
async fn migrate(pool: Pool<MySql>) -> Result<(), anyhow::Error> {
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .map_err(|e| anyhow!("{}", e))
}
async fn init_super_user(pool: Pool<MySql>) -> Result<(), sqlx::Error> {
    let count = sqlx::query("SELECT COUNT(*) FROM user WHERE user_authority = ?")
        .bind(SUPER_ADMIN_AUTHORITY)
        .map(|row: MySqlRow| row.try_get::<i64, _>(0))
        .fetch_one(&pool)
        .await??;

    let uuid = Uuid::new_v4().to_string();
    if count == 0 {
        sqlx::query("INSERT INTO user (user_account, user_password, user_authority, user_id) VALUES ($1, $2, $3, $4)")
                .bind("admin".to_string())
                .bind("zc12345679".to_string())
                .bind(SUPER_ADMIN_AUTHORITY)
                .bind(uuid)
                .execute(&pool)
                .await?;
        info!("Super User has been created");
    } else {
        info!("Super User Exists");
    }
    Ok(())
}
