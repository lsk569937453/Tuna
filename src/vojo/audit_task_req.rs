use std::collections::HashMap;

use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct AuditTaskReq {
    pub task_id: i32,
    pub source_primary_key: String,
    pub destination_primary_key: String,
}
