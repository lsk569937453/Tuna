use crate::util;
use serde::{Deserialize, Serialize};
use sqlx::types::chrono::DateTime;
use sqlx::types::chrono::Utc;
use sqlx::{Error, MySqlPool};

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct AuditTaskDao {
    pub id: i32,
    pub task_id: i32,
    pub status: i32,
    #[serde(with = "util")]
    pub timestamp: DateTime<Utc>,
}

impl AuditTaskDao {
    pub async fn create_auit_task(pool: &MySqlPool, task_id: i32) -> Result<i32, Error> {
        let result = sqlx::query(
            r#"
            INSERT INTO audit_task (task_id)
            VALUES (?)
            "#,
        )
        .bind(task_id)
        .execute(pool)
        .await?;

        Ok(result.last_insert_id() as i32)
    }

    pub async fn get_auit_task_by_id(
        pool: &MySqlPool,
        id: i32,
    ) -> Result<Option<AuditTaskDao>, Error> {
        let result = sqlx::query_as(
            r#"
            SELECT * FROM audit_task WHERE id = ?
            "#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        Ok(result)
    }
    pub async fn fetch_all_audit_tasks(pool: &MySqlPool) -> Result<Vec<AuditTaskDao>, Error> {
        let datasources = sqlx::query_as("SELECT * FROM audit_task")
            .fetch_all(pool)
            .await?;

        Ok(datasources)
    }
    pub async fn get_auit_task_by_task_id(
        pool: &MySqlPool,
        task_id: i32,
    ) -> Result<Option<AuditTaskDao>, Error> {
        let result = sqlx::query_as(
            r#"
            SELECT * FROM audit_task WHERE task_id = ?
            "#,
        )
        .bind(task_id)
        .fetch_optional(pool)
        .await?;

        Ok(result)
    }

    pub async fn update_auit_task_status(
        pool: &MySqlPool,
        id: i32,
        status: i32,
    ) -> Result<(), Error> {
        sqlx::query(
            r#"
            UPDATE audit_task SET status = ? WHERE id = ?
            "#,
        )
        .bind(status)
        .bind(id)
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn delete_audit_task(pool: &MySqlPool, id: i32) -> Result<u64, Error> {
        let res = sqlx::query(
            r#"
            DELETE FROM audit_task WHERE id = ?
            "#,
        )
        .bind(id)
        .execute(pool)
        .await?;

        Ok(res.rows_affected())
    }
}
