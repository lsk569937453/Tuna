use crate::dao::audit_task_result_clickhouse_dao::{
    AuditTaskResultClickhouseDao, AuditTaskResultStatus,
};
use crate::util::serialize_human_readable_time;

use serde::Serialize;
use time::OffsetDateTime;
#[derive(Debug, Serialize)]
pub struct AuditTaskResultResponse {
    pub id: u128,
    pub audit_task_id: u32,
    pub execution_id: String,
    pub primary_id: String,
    pub left_compare: String,
    pub right_compare: String,
    pub is_same: AuditTaskResultStatus,

    #[serde(serialize_with = "serialize_human_readable_time")]
    pub timestamp: OffsetDateTime,
}

impl From<AuditTaskResultClickhouseDao> for AuditTaskResultResponse {
    fn from(dao: AuditTaskResultClickhouseDao) -> Self {
        AuditTaskResultResponse {
            id: dao.id,
            audit_task_id: dao.audit_task_id,
            execution_id: dao.execution_id,
            primary_id: dao.primary_id,
            left_compare: dao.left_compare,
            right_compare: dao.right_compare,
            is_same: dao.is_same,
            timestamp: dao.timestamp,
        }
    }
}
