use serde::Serialize;
use serde_json::Value;

#[derive(Serialize, Clone)]
pub struct AuditTaskRes {
    pub left_compare: Vec<Value>,
    pub right_compare: Vec<Value>,
}
