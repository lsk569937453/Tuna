use crate::util;
use chrono::Utc;
use serde::Serialize;
use sqlx::types::chrono::DateTime;
#[derive(Serialize, Clone)]
pub struct GetDatasourceListResponse {
    pub datasource_name: String,
    pub addr: String,
    #[serde(with = "util")]
    pub timestamp: DateTime<Utc>,
}
