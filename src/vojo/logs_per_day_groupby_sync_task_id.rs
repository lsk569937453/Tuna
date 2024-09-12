use serde::Serialize;

#[derive(Serialize, Clone)]
pub struct LogsPerDayGroupbySyncTaskIdRes {
    pub all_days: Vec<String>,
    pub list: Vec<LogsPerDayGroupbySyncTaskIdItem>,
}

impl LogsPerDayGroupbySyncTaskIdRes {
    pub fn new(all_days: Vec<String>, list: Vec<LogsPerDayGroupbySyncTaskIdItem>) -> Self {
        Self { all_days, list }
    }
}
#[derive(Serialize, Clone)]

pub struct LogsPerDayGroupbySyncTaskIdItem {
    // #[serde(serialize_with = "serialize_human_readable_time")]
    pub sync_task_name: String,
    pub total_logs: Vec<u64>,
}
impl LogsPerDayGroupbySyncTaskIdItem {
    pub fn new(sync_task_name: String, total_logs: Vec<u64>) -> Self {
        Self {
            sync_task_name,
            total_logs,
        }
    }
}
