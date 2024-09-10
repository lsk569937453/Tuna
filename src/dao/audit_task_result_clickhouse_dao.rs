use crate::util::serialize_human_readable_time;
use clickhouse::{Client, Row};
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::fmt::Debug;
use time::OffsetDateTime;
use uuid::Uuid;

// Define the struct corresponding to your table schema
#[derive(Debug, Serialize, Deserialize, Row)]
pub struct AuditTaskResultClickhouseDao {
    pub id: u128,
    pub audit_task_id: u32,
    pub execution_id: String,
    pub primary_id: String,
    pub left_compare: String,
    pub right_compare: String,
    pub is_same: AuditTaskResultStatus,
    #[serde(
        skip_serializing,
        deserialize_with = "clickhouse::serde::time::datetime::deserialize"
    )]
    pub timestamp: OffsetDateTime,
}
#[repr(u32)]
#[derive(Serialize_repr, Deserialize_repr, Debug)]

pub enum AuditTaskResultStatus {
    Same = 0,
    Different = 1,
}
#[derive(Debug, Serialize, Deserialize, Row)]

pub struct AuditTaskResultListDao {
    pub execution_id: String,
    pub total_tasks: u64,
    pub is_same: AuditTaskResultStatus,
    #[serde(
        serialize_with = "serialize_human_readable_time",
        deserialize_with = "clickhouse::serde::time::datetime::deserialize"
    )]
    pub first_occurrence: OffsetDateTime,
}

impl AuditTaskResultClickhouseDao {
    pub fn new(
        left_compare: String,
        right_compare: String,
        audit_task_id: u32,
        execution_id: String,
        primary_id: String,
        is_same: AuditTaskResultStatus,
    ) -> Self {
        Self {
            id: Uuid::new_v4().as_u128(),
            execution_id,
            primary_id,
            audit_task_id,
            left_compare,
            right_compare,
            timestamp: OffsetDateTime::now_utc(),
            is_same,
        }
    }
    pub async fn insert_batch(
        client: Client,
        record: Vec<AuditTaskResultClickhouseDao>,
    ) -> Result<(), anyhow::Error> {
        if record.is_empty() {
            return Ok(());
        }
        info!("insert batch{}", record.len());
        let mut insert = client.insert("audit_task_result")?;
        for item in record {
            info!("insert {:?}", item);
            insert.write(&item).await?;
        }
        insert.end().await?;
        Ok(())
    }

    async fn get_by_id(
        client: Client,
        id: u32,
    ) -> Result<Option<AuditTaskResultClickhouseDao>, anyhow::Error> {
        let result = client
            .query("SELECT * FROM audit_task_result WHERE id = ?")
            .bind(id)
            .fetch_optional::<AuditTaskResultClickhouseDao>()
            .await?;
        Ok(result)
    }
    pub async fn get_by_audit_task_id(
        client: Client,
        audit_task_id: u32,
    ) -> Result<Vec<AuditTaskResultClickhouseDao>, anyhow::Error> {
        let result = client
            .query(
                "SELECT * FROM audit_task_result WHERE audit_task_id  = ? order by timestamp desc",
            )
            .bind(audit_task_id)
            .fetch_all::<AuditTaskResultClickhouseDao>()
            .await?;
        Ok(result)
    }

    pub async fn get_audit_tasks_result_list(
        client: Client,
    ) -> Result<Vec<AuditTaskResultListDao>, anyhow::Error> {
        let result = client
            .query(
                "SELECT
    execution_id,
    COUNT(*) AS total_tasks,
        MAX(is_same) AS is_same,
    MIN(timestamp) AS first_occurrence
FROM
    tuna.audit_task_result
GROUP BY
    execution_id
ORDER BY
    first_occurrence DESC",
            )
            .fetch_all::<AuditTaskResultListDao>()
            .await?;
        Ok(result)
    }
}
