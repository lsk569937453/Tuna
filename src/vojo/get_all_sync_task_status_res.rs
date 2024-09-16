use crate::util;
use crate::vojo::sync_task_status_res::SyncTaskStatus;
use chrono::Utc;
use serde::Serialize;
use sqlx::types::chrono::DateTime;
#[derive(Serialize, Clone)]
pub struct GetAllSyncTaskStatusResponse {
    pub list: Vec<GetAllSyncTaskStatusResponseItem>,
}
#[derive(Serialize, Clone)]
pub struct GetAllSyncTaskStatusResponseItem {
    pub task_name: String,
    pub status: SyncTaskStatus,
}
