use crate::util::serialize_human_readable_time;
use clickhouse::Client;
use clickhouse::Row;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;
#[derive(Serialize, Deserialize, Debug, Row)]
pub struct SqlLogDao {
    pub id: u128,
    pub sync_task_id: u32,

    pub query: String,
    pub result: String,
    pub execution_time: u64,
    pub client_ip: String,
    #[serde(
        deserialize_with = "clickhouse::serde::time::datetime64::millis::deserialize",
        serialize_with = "clickhouse::serde::time::datetime64::millis::serialize"
    )]
    pub sql_timestamp: OffsetDateTime,
    #[serde(
        skip_serializing,
        deserialize_with = "clickhouse::serde::time::datetime64::millis::deserialize"
    )]
    pub timestamp: OffsetDateTime,
}

#[derive(Serialize, Deserialize, Debug, Row)]
pub struct SqlLogResponse {
    pub id: u128,
    pub sync_task_id: u32,

    pub query: String,
    pub result: String,
    pub execution_time: u64,
    pub client_ip: String,

    #[serde(
        serialize_with = "serialize_human_readable_time",
        deserialize_with = "clickhouse::serde::time::datetime64::millis::deserialize"
    )]
    pub sql_timestamp: OffsetDateTime,

    #[serde(
        serialize_with = "serialize_human_readable_time",
        deserialize_with = "clickhouse::serde::time::datetime64::millis::deserialize"
    )]
    pub timestamp: OffsetDateTime,
}
#[derive(Debug, Serialize, Deserialize, Row)]
pub struct LogsPerMinuteDao {
    pub minute: String,
    pub total_logs: u64,
}
#[derive(Debug, Serialize, Deserialize, Row)]
pub struct LogsPerMinuteGroupbySyncTaskIdDao {
    pub minute: String,
    pub sync_task_id: u32,

    pub total_logs: u64,
}
#[derive(Debug, Serialize, Deserialize, Row)]
pub struct LogsPerDayDao {
    pub day: String,
    pub total_logs: u64,
}
#[derive(Debug, Serialize, Deserialize, Row)]
pub struct LogsPerDayGroupbySyncTaskIdDao {
    pub day: String,
    pub sync_task_id: u32,
    pub total_logs: u64,
}

impl SqlLogDao {
    pub fn new(
        query: String,
        result: String,
        execution_time: u64,
        client_ip: String,
        sql_timestamp: OffsetDateTime,
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
            sql_timestamp,
        }
    }
    pub async fn query_logs(
        client: Client,
        sync_task_id: Option<u32>,
        start_time: Option<String>,
        end_time: Option<String>,
    ) -> Result<Vec<SqlLogResponse>, anyhow::Error> {
        // Start building the base SQL query
        let mut query = String::from("SELECT * FROM sql_logs WHERE 1=1");

        // Append conditions based on the provided options
        if let Some(task_id) = sync_task_id {
            query.push_str(&format!(" AND sync_task_id = {}", task_id));
        }

        if let Some(start) = start_time {
            query.push_str(&format!(" AND sql_timestamp >= '{}'", start));
        }

        if let Some(end) = end_time {
            query.push_str(&format!(" AND sql_timestamp <= '{}'", end));
        }
        query.push_str(" ORDER BY timestamp DESC limit 1000");

        // Execute the query
        let result = client.query(&query).fetch_all::<SqlLogResponse>().await?;

        info!("query_logs: {:?}", result);

        Ok(result)
    }
    pub async fn get_logs_per_minute(
        client: Client,
    ) -> Result<Vec<LogsPerMinuteDao>, anyhow::Error> {
        let result = client
            .query(
                "SELECT formatDateTime (
        toStartOfMinute (timestamp), '%R'
    ) AS minute, count() AS total_logs
FROM tuna.sql_logs
WHERE
    timestamp >= toStartOfDay (now())
GROUP BY
    minute
ORDER BY minute",
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
    formatDateTime (
        toStartOfMinute (timestamp), '%R'
    ) AS minute, 
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
                "SELECT formatDateTime (
        toStartOfDay (timestamp), '%F'
    ) AS day, count() AS total_logs
FROM tuna.sql_logs
WHERE
    timestamp >= subtractDays (now(), 30)
GROUP BY
    day
ORDER BY day",
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
                "SELECT formatDateTime (
        toStartOfDay (timestamp), '%F'
    ) AS day,
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
