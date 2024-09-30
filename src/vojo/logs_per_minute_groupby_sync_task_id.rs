
use indexmap::IndexMap;
use serde::Serialize;

#[derive(Serialize, Clone)]
pub struct LogsPerminuteGroupbySyncTaskIdRes {
    pub list: Vec<LogsPerminuteGroupbySyncTaskIdResItem>,
}
impl LogsPerminuteGroupbySyncTaskIdRes {
    pub fn new(list: Vec<LogsPerminuteGroupbySyncTaskIdResItem>) -> Self {
        Self { list }
    }
}
#[derive(Serialize, Clone)]

pub struct LogsPerminuteGroupbySyncTaskIdResItem {
    pub sync_task_name: String,
    #[serde(with = "indexmap::map::serde_seq")]
    pub total_logs: IndexMap<String, u64>,
}
impl LogsPerminuteGroupbySyncTaskIdResItem {
    pub fn new(sync_task_name: String, total_logs: IndexMap<String, u64>) -> Self {
        Self {
            sync_task_name,
            total_logs,
        }
    }
}
