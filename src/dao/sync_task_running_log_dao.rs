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
}
