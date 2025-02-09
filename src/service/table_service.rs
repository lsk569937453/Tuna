use crate::common::app_state::AppState;
use crate::dao::datasource_dao::DataSourceDao;
use crate::handle_response;
use crate::vojo::base_response::BaseResponse;
use axum::extract::Path;
use axum::extract::State;
use axum::response::IntoResponse;
use axum::response::Response;

use sqlx::Connection;
use sqlx::MySqlConnection;
use sqlx::Row;
use std::convert::Infallible;
pub async fn get_table_list(
    State(state): State<AppState>,
    Path(data): Path<i32>,
) -> Result<Response, Infallible> {
    handle_response!(get_table_list_with_error(state, data).await)
}
async fn get_table_list_with_error(
    app_state: AppState,
    datasource_id: i32,
) -> Result<String, anyhow::Error> {
    let datasource_url = DataSourceDao::find_by_id(&app_state.db_pool, datasource_id)
        .await?
        .datasource_url;
    let mut conn = MySqlConnection::connect(&datasource_url).await?;
    let sql = "show tables;";
    let sql_rows = sqlx::query(sql).fetch_all(&mut conn).await?;

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
