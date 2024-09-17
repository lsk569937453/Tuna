use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone)]
pub struct QueryLogsReq {
    pub sync_task_id: Option<u32>,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
}
