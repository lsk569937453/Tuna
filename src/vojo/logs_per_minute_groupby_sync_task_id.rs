use crate::util;
use crate::util::serialize_human_readable_time;
use chrono::Utc;
use serde::Serialize;
use sqlx::types::chrono::DateTime;
use time::OffsetDateTime;
#[derive(Serialize, Clone)]
pub struct LogsPerminuteGroupbySyncTaskId {
    pub sync_task_name: String,
    pub list: Vec<LogsPerminuteGroupbySyncTaskIdItem>,
}
#[derive(Serialize, Clone)]

pub struct LogsPerminuteGroupbySyncTaskIdItem {
    #[serde(serialize_with = "serialize_human_readable_time")]
    pub minute: OffsetDateTime,
    pub total_logs: u64,
}
impl LogsPerminuteGroupbySyncTaskIdItem {
    pub fn new(minute: OffsetDateTime, total_logs: u64) -> Self {
        Self { minute, total_logs }
    }
}
