use crate::util;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::types::chrono::DateTime;
#[derive(Serialize, Clone)]
pub struct GetDatasourceListResponse {
    pub datasource_name: String,
    pub addr: String,
    #[serde(with = "util")]
    pub timestamp: DateTime<Utc>,
}
impl GetDatasourceListResponse {
    pub fn new(datasource_name: String, addr: String, timestamp: DateTime<Utc>) -> Self {
        Self {
            datasource_name,
            addr,
            timestamp,
        }
    }
}
