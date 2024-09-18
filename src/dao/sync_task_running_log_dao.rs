use crate::util::serialize_human_readable_time;
use chrono::{DateTime, Utc};
use clickhouse::{Client, Row};
use serde::Serializer;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Row)]
pub struct SyncTaskRunningLogsDao {
    pub id: u128,
    pub sync_task_uuid: u128,
    pub level: Loglevel,
    pub message: String,
    pub sync_task_id: u32,

    #[serde(
        skip_serializing,
        deserialize_with = "clickhouse::serde::time::datetime64::millis::deserialize"
    )]
    pub timestamp: OffsetDateTime,
}
#[derive(Debug, Deserialize)]
pub enum Loglevel {
    Info,
    Error,
}

#[derive(Debug, Serialize, Deserialize, Row)]

pub struct SyncTaskSummaryByTaskIdDao {
    pub sync_task_id: u32,
    pub sync_task_uuid: u128,

    #[serde(
        serialize_with = "serialize_human_readable_time",
        deserialize_with = "clickhouse::serde::time::datetime64::millis::deserialize"
    )]
    pub latest_timestamp: OffsetDateTime,
    #[serde(
        serialize_with = "serialize_human_readable_time",
        deserialize_with = "clickhouse::serde::time::datetime64::millis::deserialize"
    )]
    pub oldest_timestamp: OffsetDateTime,
}
impl Serialize for Loglevel {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let level_str = match *self {
            Loglevel::Info => "INFO",
            Loglevel::Error => "ERROR",
        };
        serializer.serialize_str(level_str)
    }
}
impl SyncTaskRunningLogsDao {
    pub fn new(sync_task_uuid: u128, level: Loglevel, message: String, sync_task_id: u32) -> Self {
        Self {
            id: Uuid::new_v4().as_u128(),
            sync_task_uuid,
            level,
            message,
            sync_task_id,
            timestamp: OffsetDateTime::now_utc(),
        }
    }
    pub async fn insert_one(
        client: Client,
        record: SyncTaskRunningLogsDao,
    ) -> Result<(), anyhow::Error> {
        let mut insert = client.insert("sync_task_running_logs")?;
        insert.write(&record).await?;
        insert.end().await?;
        Ok(())
    }
    pub async fn get_sync_task_summary_by_task_id(
        client: Client,
    ) -> Result<Vec<SyncTaskSummaryByTaskIdDao>, anyhow::Error> {
        let result = client
            .query(
                "SELECT
    sync_task_id,
    sync_task_uuid,
    latest_timestamp,
    oldest_timestamp
FROM (
    SELECT
        sync_task_id,
        sync_task_uuid,
        MAX(timestamp) AS latest_timestamp,
        MIN(timestamp) AS oldest_timestamp,
        ROW_NUMBER() OVER (PARTITION BY sync_task_id ORDER BY MAX(timestamp) DESC) AS rn
    FROM sync_task_running_logs
    GROUP BY sync_task_id, sync_task_uuid
)
WHERE rn = 1
ORDER BY oldest_timestamp DESC
",
            )
            .fetch_all::<SyncTaskSummaryByTaskIdDao>()
            .await?;
        Ok(result)
    }
}
