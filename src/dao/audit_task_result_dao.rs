use crate::util;
use serde::{Deserialize, Serialize};
use sqlx::types::chrono::DateTime;
use sqlx::types::chrono::Utc;

use sqlx::{Error, MySql, MySqlPool};
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct AuditTaskResultDao {
    pub id: i32,
    pub audit_task_id: i32,
    pub left_compare: Option<String>,
    pub right_compare: Option<String>,
    #[serde(with = "util")]
    pub timestamp: DateTime<Utc>,
}

impl AuditTaskResultDao {
    pub async fn create_task_result(
        pool: &MySqlPool,
        audit_task_id: i32,
        left_compare: Option<String>,
        right_compare: Option<String>,
    ) -> Result<i32, Error> {
        let result = sqlx::query!(
            r#"
            INSERT INTO audit_task_result (audit_task_id, left_compare, right_compare)
            VALUES (?, ?, ?)
            "#,
            audit_task_id,
            left_compare,
            right_compare
        )
        .execute(pool)
        .await?;

        Ok(result.last_insert_id() as i32)
    }

    pub async fn get_task_result_by_id(
        pool: &MySqlPool,
        id: i32,
    ) -> Result<Option<AuditTaskResultDao>, Error> {
        let result = sqlx::query_as!(
            AuditTaskResultDao,
            r#"
            SELECT * FROM audit_task_result WHERE id = ?
            "#,
            id
        )
        .fetch_optional(pool)
        .await?;

        Ok(result)
    }

    pub async fn get_task_results_by_audit_task_id(
        pool: &MySqlPool,
        audit_task_id: i32,
    ) -> Result<Vec<AuditTaskResultDao>, Error> {
        let results = sqlx::query_as!(
            AuditTaskResultDao,
            r#"
            SELECT * FROM audit_task_result WHERE audit_task_id = ?
            "#,
            audit_task_id
        )
        .fetch_all(pool)
        .await?;

        Ok(results)
    }

    pub async fn update_task_result(
        pool: &MySqlPool,
        id: i32,
        left_compare: Option<String>,
        right_compare: Option<String>,
    ) -> Result<(), Error> {
        sqlx::query!(
            r#"
            UPDATE audit_task_result SET left_compare = ?, right_compare = ? WHERE id = ?
            "#,
            left_compare,
            right_compare,
            id
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn delete_task_result(pool: &MySqlPool, id: i32) -> Result<(), Error> {
        sqlx::query!(
            r#"
            DELETE FROM audit_task_result WHERE id = ?
            "#,
            id
        )
        .execute(pool)
        .await?;

        Ok(())
    }
    pub async fn fetch_all_audit_tasks_result(
        pool: &MySqlPool,
    ) -> Result<Vec<AuditTaskResultDao>, Error> {
        let datasources = sqlx::query_as!(
            AuditTaskResultDao,
            "SELECT * FROM audit_task_result order by id desc"
        )
        .fetch_all(pool)
        .await?;

        Ok(datasources)
    }
}
