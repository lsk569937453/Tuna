use std::collections::HashMap;

use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct AuditTaskReq {
    pub task_id: i32,
    pub from_primary_key: String,
    pub to_primary_key: String,
}
