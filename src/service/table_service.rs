use crate::dao::datasource_dao::DataSourceDao;
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
pub async fn get_table_list(
    State(state): State<Pool<MySql>>,
    Path((data, database_name)): Path<(i32, String)>,
) -> Result<Response, Infallible> {
    handle_response!(get_table_list_with_error(state, data, database_name).await)
}
async fn get_table_list_with_error(
    pool: Pool<MySql>,
    datasource_id: i32,
    database_name: String,
) -> Result<String, anyhow::Error> {
    let datasource_url = DataSourceDao::find_by_id(&pool, datasource_id)
        .await?
        .datasource_url;
    let mut conn = MySqlConnection::connect(&datasource_url).await?;
    let sql = format!("show tables in {}", database_name);
    let sql_rows = sqlx::query(&sql).fetch_all(&mut conn).await?;

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
