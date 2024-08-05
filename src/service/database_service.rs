use crate::handle_response;
use crate::vojo::base_response::BaseResponse;
use crate::vojo::create_datasource_req;
use crate::vojo::create_datasource_req::CreateDatasourceReq;
use axum::extract::Path;
use axum::extract::State;
use axum::http::header;
use axum::response::IntoResponse;
use axum::response::Response;
use axum::Json;
use clap::builder::Str;
use sqlx::mysql::MySqlConnectOptions;
use sqlx::mysql::MySqlRow;
use sqlx::Connection;
use sqlx::MySqlConnection;
use sqlx::MySqlPool;
use sqlx::Row;
use sqlx::{MySql, Pool};
use std::convert::Infallible;
use std::str::FromStr;
pub async fn get_database_list(
    State(state): State<Pool<MySql>>,
    Path(data): Path<i32>,
) -> Result<Response, Infallible> {
    handle_response!(get_database_list_with_error(state, data).await)
}
async fn get_database_list_with_error(
    pool: Pool<MySql>,
    datasource_id: i32,
) -> Result<String, anyhow::Error> {
    let datasource_url = sqlx::query("SELECT datasource_url FROM datasource WHERE id = ?")
        .bind(datasource_id)
        .map(|row: MySqlRow| row.try_get::<String, _>(0))
        .fetch_one(&pool)
        .await??;
    let mut conn = MySqlConnection::connect(&datasource_url).await?;
    let sql_rows= sqlx::query(
        "SHOW DATABASES WHERE `Database` NOT IN ('mysql', 'performance_schema', 'sys','information_schema')",
    )
    .fetch_all(&mut conn)
    .await?;

    let mut res = vec![];
    for it in sql_rows.iter() {
        let item: Vec<u8> = it.try_get::<Vec<u8>, _>(0)?;
        res.push(String::from_utf8(item)?);
    }
    let data = BaseResponse {
        response_code: 0,
        response_object: res,
    };

    serde_json::to_string(&data).map_err(|e| anyhow!("{}", e))
}
