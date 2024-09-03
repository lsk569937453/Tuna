use clickhouse::{Client, Row};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// Define the struct corresponding to your table schema
#[derive(Debug, Serialize, Deserialize, Row)]
pub struct AuditTaskResultClickhouseDao {
    pub id: u128,
    pub audit_task_id: u32,
    pub execution_id: String,
    pub primary_id: String,
    pub left_compare: String,
    pub right_compare: String,
    #[serde(skip_serializing)]
    pub timestamp: chrono::NaiveDateTime,
}
impl AuditTaskResultClickhouseDao {
    pub fn new(
        left_compare: String,
        right_compare: String,
        audit_task_id: u32,
        execution_id: String,
        primary_id: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4().as_u128(),
            execution_id,
            primary_id,
            audit_task_id,
            left_compare,
            right_compare,
            timestamp: chrono::Utc::now().naive_utc(),
        }
    }
    pub async fn insert_batch(
        client: Client,
        record: Vec<AuditTaskResultClickhouseDao>,
    ) -> Result<(), anyhow::Error> {
        info!("insert batch{}", record.len());
        let mut insert = client.insert("audit_task_result")?;
        for item in record {
            insert.write(&item).await?;
        }
        insert.end().await?;
        Ok(())
    }

    async fn get_by_id(
        client: Client,
        id: u32,
    ) -> Result<Option<AuditTaskResultClickhouseDao>, anyhow::Error> {
        let result = client
            .query("SELECT * FROM audit_task_result WHERE id = ?")
            .bind(id)
            .fetch_optional::<AuditTaskResultClickhouseDao>()
            .await?;
        Ok(result)
    }
}
