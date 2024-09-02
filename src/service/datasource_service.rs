use crate::common::app_state::AppState;
use crate::dao::datasource_dao::DataSourceDao;
use crate::handle_response;
use crate::vojo::base_response::BaseResponse;
use crate::vojo::create_datasource_req::CreateDatasourceReq;
use axum::extract::Path;
use axum::extract::State;
use axum::response::IntoResponse;
use axum::response::Response;
use axum::Json;
use sqlx::mysql::MySqlConnectOptions;
use sqlx::{MySql, Pool};
use std::convert::Infallible;
use std::str::FromStr;
pub async fn create_datasource(
    State(state): State<AppState>,
    Json(data): Json<CreateDatasourceReq>,
) -> Result<Response, Infallible> {
    handle_response!(create_datasource_with_error(state, data).await)
}
async fn create_datasource_with_error(
    app_state: AppState,
    create_datasource_req: CreateDatasourceReq,
) -> Result<String, anyhow::Error> {
    let options = MySqlConnectOptions::from_str(&create_datasource_req.datasource_url)?;
    let host = options.get_host();
    let port = options.get_port();

    DataSourceDao::create(
        &app_state.db_pool,
        create_datasource_req.datasource_name,
        create_datasource_req.datasource_url,
        host.to_string(),
        port as i32,
    )
    .await?;
    let data = BaseResponse {
        response_code: 0,
        response_object: 0,
    };
    serde_json::to_string(&data).map_err(|e| anyhow!("{}", e))
}
pub async fn get_datasource_list(State(state): State<AppState>) -> Result<Response, Infallible> {
    handle_response!(get_datasource_list_with_error(state).await)
}
async fn get_datasource_list_with_error(app_state: AppState) -> Result<String, anyhow::Error> {
    let redis_util = DataSourceDao::fetch_all_datasources(&app_state.db_pool).await?;
    let data = BaseResponse {
        response_code: 0,
        response_object: redis_util,
    };
    serde_json::to_string(&data).map_err(|e| anyhow!("{}", e))
}
pub async fn delete_datasource_by_id(
    State(state): State<AppState>,
    Path(data): Path<i32>,
) -> Result<Response, Infallible> {
    handle_response!(delete_datasource_by_id_with_error(state, data).await)
}
async fn delete_datasource_by_id_with_error(
    app_state: AppState,
    id: i32,
) -> Result<String, anyhow::Error> {
    let redis_util = DataSourceDao::delete(&app_state.db_pool, id).await?;
    let data = BaseResponse {
        response_code: 0,
        response_object: redis_util,
    };
    serde_json::to_string(&data).map_err(|e| anyhow!("{}", e))
}
