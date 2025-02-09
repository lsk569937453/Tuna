use crate::util;
use serde::Serialize;
use sqlx::mysql::MySqlPool;
use sqlx::types::chrono::DateTime;
use sqlx::types::chrono::Utc;
use sqlx::Error;
use sqlx::FromRow;

#[derive(Debug, FromRow, Serialize)]
pub struct DataSourceDao {
    pub id: i32,
    pub datasource_name: String,
    pub datasource_url: String,
    pub host: String,
    pub port: i32,
    #[serde(with = "util")]
    pub timestamp: DateTime<Utc>,
}

impl DataSourceDao {
    pub async fn find_by_id(pool: &MySqlPool, id: i32) -> Result<Self, Error> {
        sqlx::query_as(
            "SELECT id, datasource_name, datasource_url, host, port, timestamp FROM datasource WHERE id = ?",
            
        ).bind(id)
        .fetch_one(pool)
        .await
    }

    pub async fn fetch_all_datasources(pool: &MySqlPool) -> Result<Vec<DataSourceDao>, Error> {
        let datasources = sqlx::query_as( "SELECT * FROM datasource")
            .fetch_all(pool)
            .await?;

        Ok(datasources)
    }
    pub async fn create(
        pool: &MySqlPool,
        datasource_name: String,
        datasource_url: String,
        host: String,
        port: i32,
    ) -> Result<(), Error> {
        sqlx::query(
            "INSERT INTO datasource (datasource_name, datasource_url, host, port) VALUES (?,?,?,?)",
        ).bind(datasource_name).bind(datasource_url).bind(host).bind(port)
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn delete(pool: &MySqlPool, id: i32) -> Result<(), Error> {
        sqlx::query("DELETE FROM datasource WHERE id = ?").bind(id)
            .execute(pool)
            .await?;
        Ok(())
    }
}
