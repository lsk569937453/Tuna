use crate::util::serialize_human_readable_time;
use clickhouse::Client;
use clickhouse::Row;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Row)]
pub struct SqlLogDao {
    pub id: u128,
    pub query: String,
    pub result: String,
    pub execution_time: u64,
    pub client_ip: String,
    pub sync_task_id: u32,
    #[serde(
        skip_serializing,
        deserialize_with = "clickhouse::serde::time::datetime::deserialize"
    )]
    pub timestamp: OffsetDateTime,
}
#[derive(Debug, Serialize, Deserialize, Row)]
pub struct LogsPerMinuteDao {
    #[serde(
        serialize_with = "serialize_human_readable_time",
        deserialize_with = "clickhouse::serde::time::datetime::deserialize"
    )]
    pub minute: OffsetDateTime,
    pub total_logs: u64,
}
#[derive(Debug, Serialize, Deserialize, Row)]
pub struct LogsPerMinuteGroupbySyncTaskIdDao {
    #[serde(
        serialize_with = "serialize_human_readable_time",
        deserialize_with = "clickhouse::serde::time::datetime::deserialize"
    )]
    pub minute: OffsetDateTime,
    pub sync_task_id: u32,

    pub total_logs: u64,
}
#[derive(Debug, Serialize, Deserialize, Row)]
pub struct LogsPerDayDao {
    #[serde(
        serialize_with = "serialize_human_readable_time",
        deserialize_with = "clickhouse::serde::time::datetime::deserialize"
    )]
    pub day: OffsetDateTime,
    pub total_logs: u64,
}
#[derive(Debug, Serialize, Deserialize, Row)]
pub struct LogsPerDayGroupbySyncTaskIdDao {
    #[serde(
        serialize_with = "serialize_human_readable_time",
        deserialize_with = "clickhouse::serde::time::datetime::deserialize"
    )]
    pub day: OffsetDateTime,
    pub sync_task_id: u32,
    pub total_logs: u64,
}

impl SqlLogDao {
    pub fn new(
        query: String,
        result: String,
        execution_time: u64,
        client_ip: String,
        sync_task_id: u32,
    ) -> Self {
        Self {
            id: Uuid::new_v4().as_u128(),
            query,
            result,
            execution_time,
            timestamp: OffsetDateTime::now_utc(),
            client_ip,
            sync_task_id,
        }
    }

    pub async fn get_logs_per_minute(
        client: Client,
    ) -> Result<Vec<LogsPerMinuteDao>, anyhow::Error> {
        let result = client
            .query(
                "SELECT 
    toStartOfMinute(timestamp) AS minute, 
    count() AS total_logs
FROM 
    tuna.sql_logs
WHERE 
    timestamp >= toStartOfDay(now())
GROUP BY 
    minute
ORDER BY 
    minute",
            )
            .fetch_all::<LogsPerMinuteDao>()
            .await?;

        Ok(result)
    }
    pub async fn get_logs_per_minute_groupby_sync_task_id(
        client: Client,
    ) -> Result<Vec<LogsPerMinuteGroupbySyncTaskIdDao>, anyhow::Error> {
        let result = client
            .query(
                "SELECT 
    toStartOfMinute(timestamp) AS minute, 
    sync_task_id,  
    count() AS total_logs
FROM 
    tuna.sql_logs
WHERE 
    timestamp >= toStartOfDay(now())
GROUP BY 
    minute, sync_task_id
ORDER BY 
    minute, sync_task_id
",
            )
            .fetch_all::<LogsPerMinuteGroupbySyncTaskIdDao>()
            .await?;

        Ok(result)
    }
    pub async fn get_logs_per_day(client: Client) -> Result<Vec<LogsPerDayDao>, anyhow::Error> {
        let result = client
            .query(
                "SELECT 
    toStartOfDay(timestamp) AS day, 
    count() AS total_logs
FROM 
    tuna.sql_logs
WHERE 
    timestamp >= subtractDays(now(), 30)  
GROUP BY 
    day
ORDER BY 
    day",
            )
            .fetch_all::<LogsPerDayDao>()
            .await?;

        Ok(result)
    }
    pub async fn get_logs_per_day_groupby_sync_task_id(
        client: Client,
    ) -> Result<Vec<LogsPerDayGroupbySyncTaskIdDao>, anyhow::Error> {
        let result = client
            .query(
                "SELECT 
    toStartOfDay(timestamp) AS day, 
    sync_task_id,
    count() AS total_logs
FROM 
    tuna.sql_logs
WHERE 
    timestamp >= subtractDays(now(), 30)  
GROUP BY 
    day, 
    sync_task_id
ORDER BY 
    day, 
    sync_task_id
",
            )
            .fetch_all::<LogsPerDayGroupbySyncTaskIdDao>()
            .await?;

        Ok(result)
    }
}
