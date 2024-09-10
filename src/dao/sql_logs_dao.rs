use clickhouse::inserter::Inserter;
use clickhouse::Row;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Row)]
pub struct SqlLogDao {
    pub id: u128,
    pub query: String,
    pub result: String,
    pub execution_time: u64,
    #[serde(
        skip_serializing,
        deserialize_with = "clickhouse::serde::time::datetime::deserialize"
    )]
    pub timestamp: OffsetDateTime,
}
impl SqlLogDao {
    pub fn new(query: String, result: String, execution_time: u64) -> Self {
        Self {
            id: Uuid::new_v4().as_u128(),
            query,
            result,
            execution_time,
            timestamp: OffsetDateTime::now_utc(),
        }
    }
    pub async fn insert_infinite_sql_log(
        inserter: &mut Inserter<SqlLogDao>,
        record: SqlLogDao,
    ) -> Result<(), anyhow::Error> {
        inserter.write(&record).await?;
        Ok(())
    }
}
