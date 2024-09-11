use crate::util;
use crate::util::serialize_human_readable_time;
use chrono::Utc;
use serde::Serialize;
use sqlx::types::chrono::DateTime;
use time::OffsetDateTime;
#[derive(Serialize, Clone)]
pub struct LogsPerminuteGroupbySyncTaskIdRes {
    pub all_minutes: Vec<String>,
    pub list: Vec<LogsPerminuteGroupbySyncTaskIdResItem>,
}
impl LogsPerminuteGroupbySyncTaskIdRes {
    pub fn new(all_minutes: Vec<String>, list: Vec<LogsPerminuteGroupbySyncTaskIdResItem>) -> Self {
        Self { all_minutes, list }
    }
}
#[derive(Serialize, Clone)]

pub struct LogsPerminuteGroupbySyncTaskIdResItem {
    pub sync_task_name: String,
    pub total_logs: Vec<u64>,
}
impl LogsPerminuteGroupbySyncTaskIdResItem {
    pub fn new(sync_task_name: String, total_logs: Vec<u64>) -> Self {
        Self {
            sync_task_name,
            total_logs,
        }
    }
}
