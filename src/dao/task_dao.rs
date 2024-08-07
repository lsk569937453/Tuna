use crate::util;
use serde::Serialize;
use sqlx::mysql::MySqlPool;
use sqlx::types::chrono::DateTime;
use sqlx::types::chrono::Utc;
use sqlx::Error;
use sqlx::FromRow;

use crate::vojo::create_task_req::CreateTaskReq;
#[derive(Debug, FromRow, Serialize)]
pub struct TaskDao {
    pub id: i32,
    pub task_name: String,
    pub from_datasource_id: i32,
    pub to_datasource_id: i32,
    pub source_database_name: String,
    pub destination_database_name: String,
    pub source_table_name: String,
    pub destination_table_name: String,
    pub status: i32,
    pub worker_ip: String,
    pub binlog_name: String,
    pub offset: String,
    #[serde(with = "util")]
    pub timestamp: DateTime<Utc>,
}

impl TaskDao {
    pub async fn create_task(pool: &MySqlPool, task: &CreateTaskReq) -> Result<(), anyhow::Error> {
        sqlx::query_as!(
            TaskDao,
            r#"
            INSERT INTO task (
                task_name,
                from_datasource_id,
                to_datasource_id,
                source_database_name,
                destination_database_name,
                source_table_name,
                destination_table_name
            ) VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
            task.task_name,
            task.from_datasource_id,
            task.to_datasource_id,
            task.source_database_name,
            task.destination_database_name,
            task.source_table_name,
            task.destination_table_name,
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn get_task(pool: &MySqlPool, task_id: i32) -> Result<TaskDao, anyhow::Error> {
        let task = sqlx::query_as!(
            TaskDao,
            r#"
            SELECT
                id,
                task_name,
                from_datasource_id,
                to_datasource_id,
                source_database_name,
                destination_database_name,
                source_table_name,
                destination_table_name,
                status,
                worker_ip,
                binlog_name,
                offset,
                timestamp
            FROM task
            WHERE id = ?
            "#,
            task_id
        )
        .fetch_one(pool)
        .await?;

        Ok(task)
    }
    pub async fn fetch_all_datasources(pool: &MySqlPool) -> Result<Vec<TaskDao>, Error> {
        let datasources = sqlx::query_as!(TaskDao, "SELECT * FROM task")
            .fetch_all(pool)
            .await?;

        Ok(datasources)
    }
    pub async fn update_task(pool: &MySqlPool, task: &TaskDao) -> Result<(), anyhow::Error> {
        sqlx::query!(
            r#"
            UPDATE task SET
                task_name = ?,
                from_datasource_id = ?,
                to_datasource_id = ?,
                source_database_name = ?,
                destination_database_name = ?,
                source_table_name = ?,
                destination_table_name = ?,
                status = ?,
                worker_ip = ?,
                binlog_name = ?,
                offset = ?,
                timestamp = ?
            WHERE id = ?
            "#,
            task.task_name,
            task.from_datasource_id,
            task.to_datasource_id,
            task.source_database_name,
            task.destination_database_name,
            task.source_table_name,
            task.destination_table_name,
            task.status,
            task.worker_ip,
            task.binlog_name,
            task.offset,
            task.timestamp,
            task.id
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn delete_task(pool: &MySqlPool, task_id: i32) -> Result<(), anyhow::Error> {
        sqlx::query!(
            r#"
            DELETE FROM task WHERE id = ?
            "#,
            task_id
        )
        .execute(pool)
        .await?;

        Ok(())
    }
}
