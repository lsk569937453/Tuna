use serde::Serialize;

#[derive(Serialize, Clone)]
pub struct MysqlExecutionResult {
    pub affected_rows: u64,
    pub last_insert_id: Option<u64>,
}
impl MysqlExecutionResult {
    pub fn new(affected_rows: u64, last_insert_id: Option<u64>) -> Self {
        Self {
            affected_rows,
            last_insert_id,
        }
    }
    pub fn as_string(&self) -> Result<String, anyhow::Error> {
        serde_json::to_string(&self).map_err(|e| anyhow!(e))
    }
}
