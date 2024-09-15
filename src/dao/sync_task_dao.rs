use crate::util;
use serde::Serialize;
use sqlx::mysql::MySqlPool;
use sqlx::types::chrono::DateTime;
use sqlx::types::chrono::Utc;
use sqlx::Error;
use sqlx::FromRow;

use crate::vojo::create_audit_task_req::CreateTaskReq;
#[derive(Debug, FromRow, Serialize, Clone)]
pub struct SyncTaskDao {
    pub id: i32,
    pub task_name: String,
    pub from_datasource_id: i32,
    pub to_datasource_id: i32,
    pub from_datasource_url: String,
    pub to_datasource_url: String,
    pub from_database_name: String,
    pub to_database_name: String,
    pub table_mapping: String,
    #[serde(with = "util")]
    pub timestamp: DateTime<Utc>,
}

impl SyncTaskDao {
    pub async fn create_task(
        pool: &MySqlPool,
        task: &CreateTaskReq,
        from_datasource_url: String,
        to_datasource_url: String,
    ) -> Result<(), anyhow::Error> {
        let table_mapping = serde_json::to_string(&task.table_mapping)?;
        sqlx::query(
            r#"
            INSERT INTO sync_task (
                task_name,
                from_datasource_id,
                to_datasource_id,
                from_database_name,
                to_database_name,
                table_mapping,
                from_datasource_url,
                to_datasource_url
            ) VALUES (?, ?, ?, ?, ?, ?,?,?)
            "#,
        )
        .bind(task.task_name.clone())
        .bind(task.from_datasource_id)
        .bind(task.to_datasource_id)
        .bind(task.from_database_name.clone())
        .bind(task.to_database_name.clone())
        .bind(table_mapping)
        .bind(from_datasource_url)
        .bind(to_datasource_url)
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn get_task(
        pool: &MySqlPool,
        task_id: i32,
    ) -> Result<Option<SyncTaskDao>, anyhow::Error> {
        let task = sqlx::query_as(
            r#"
            SELECT
                id,
                task_name,
                from_datasource_id,
                to_datasource_id,
                from_datasource_url,
                to_datasource_url,
                from_database_name,
                to_database_name,
                table_mapping,
                timestamp
            FROM sync_task
            WHERE id = ?
            "#,
        )
        .bind(task_id)
        .fetch_optional(pool)
        .await?;

        Ok(task)
    }
    pub async fn fetch_all_tasks(pool: &MySqlPool) -> Result<Vec<SyncTaskDao>, Error> {
        let tasks = sqlx::query_as("SELECT * FROM sync_task")
            .fetch_all(pool)
            .await?;

        Ok(tasks)
    }
    pub async fn update_task(pool: &MySqlPool, task: &SyncTaskDao) -> Result<(), anyhow::Error> {
        sqlx::query(
            r#"
            UPDATE sync_task SET
                task_name = ?,
                from_datasource_id = ?,
                to_datasource_id = ?,
                from_database_name = ?,
                to_database_name = ?,
                table_mapping = ?,
                timestamp = ?
            WHERE id = ?
            "#,
        )
        .bind(task.task_name.clone())
        .bind(task.from_datasource_id)
        .bind(task.to_datasource_id)
        .bind(task.from_database_name.clone())
        .bind(task.to_database_name.clone())
        .bind(task.table_mapping.clone())
        .bind(task.timestamp)
        .bind(task.id)
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn delete_task(pool: &MySqlPool, task_id: i32) -> Result<(), anyhow::Error> {
        sqlx::query(
            r#"
            DELETE FROM sync_task WHERE id = ?
            "#,
        )
        .bind(task_id)
        .execute(pool)
        .await?;

        Ok(())
    }
}
