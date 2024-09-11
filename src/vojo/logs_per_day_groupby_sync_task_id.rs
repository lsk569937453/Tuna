use crate::util;
use crate::util::serialize_human_readable_time;
use chrono::Utc;
use serde::Serialize;
use sqlx::types::chrono::DateTime;
use time::OffsetDateTime;
#[derive(Serialize, Clone)]
pub struct LogsPerDayGroupbySyncTaskId {
    pub sync_task_name: String,
    pub list: Vec<LogsPerDayGroupbySyncTaskIdItem>,
}
#[derive(Serialize, Clone)]

pub struct LogsPerDayGroupbySyncTaskIdItem {
    #[serde(serialize_with = "serialize_human_readable_time")]
    pub day: OffsetDateTime,
    pub total_logs: u64,
}
impl LogsPerDayGroupbySyncTaskIdItem {
    pub fn new(day: OffsetDateTime, total_logs: u64) -> Self {
        Self { day, total_logs }
    }
}
