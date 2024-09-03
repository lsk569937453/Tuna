use clickhouse::{Client, Row};
use serde::{Deserialize, Serialize};

// Define the struct corresponding to your table schema
#[derive(Debug, Serialize, Deserialize, Row)]
pub struct AuditTaskResultClickhouseDao {
    pub id: u32,
    pub audit_task_id: u32,
    pub left_compare: String,
    pub right_compare: String,
    pub timestamp: chrono::NaiveDateTime,
}
impl AuditTaskResultClickhouseDao {
    async fn insert_batch(
        client: Client,
        record: Vec<AuditTaskResultClickhouseDao>,
    ) -> Result<(), anyhow::Error> {
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
