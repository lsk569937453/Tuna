use serde::{Deserialize, Serialize};
use sqlx::types::chrono::DateTime;
use sqlx::types::chrono::Utc;
use sqlx::{Error, MySql, MySqlPool};
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct AuditTaskDao {
    pub id: i32,
    pub task_id: i32,
    pub status: i32,
    pub timestamp: DateTime<Utc>,
}

impl AuditTaskDao {
    pub async fn create_task(pool: &MySqlPool, task_id: i32) -> Result<i32, Error> {
        let result = sqlx::query!(
            r#"
            INSERT INTO audit_task (task_id)
            VALUES (?)
            "#,
            task_id
        )
        .execute(pool)
        .await?;

        Ok(result.last_insert_id() as i32)
    }

    pub async fn get_task_by_id(pool: &MySqlPool, id: i32) -> Result<Option<AuditTaskDao>, Error> {
        let result = sqlx::query_as!(
            AuditTaskDao,
            r#"
            SELECT * FROM audit_task WHERE id = ?
            "#,
            id
        )
        .fetch_optional(pool)
        .await?;

        Ok(result)
    }

    pub async fn get_task_by_task_id(
        pool: &MySqlPool,
        task_id: i32,
    ) -> Result<Option<AuditTaskDao>, Error> {
        let result = sqlx::query_as!(
            AuditTaskDao,
            r#"
            SELECT * FROM audit_task WHERE task_id = ?
            "#,
            task_id
        )
        .fetch_optional(pool)
        .await?;

        Ok(result)
    }

    pub async fn update_task_status(pool: &MySqlPool, id: i32, status: i32) -> Result<(), Error> {
        sqlx::query!(
            r#"
            UPDATE audit_task SET status = ? WHERE id = ?
            "#,
            status,
            id
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn delete_task(pool: &MySqlPool, id: i32) -> Result<(), Error> {
        sqlx::query!(
            r#"
            DELETE FROM audit_task WHERE id = ?
            "#,
            id
        )
        .execute(pool)
        .await?;

        Ok(())
    }
}
