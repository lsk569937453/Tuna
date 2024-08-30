use crate::dao::datasource_dao::DataSourceDao;
use crate::handle_response;
use crate::vojo::base_response::BaseResponse;
use crate::vojo::create_datasource_req::CreateDatasourceReq;
use axum::extract::State;
use axum::response::IntoResponse;
use axum::response::Response;
use axum::Json;
use sqlx::mysql::MySqlConnectOptions;
use sqlx::{MySql, Pool};
use std::convert::Infallible;
use std::str::FromStr;
pub async fn create_datasource(
    State(state): State<Pool<MySql>>,
    Json(data): Json<CreateDatasourceReq>,
) -> Result<Response, Infallible> {
    handle_response!(create_datasource_with_error(state, data).await)
}
async fn create_datasource_with_error(
    pool: Pool<MySql>,
    create_datasource_req: CreateDatasourceReq,
) -> Result<String, anyhow::Error> {
    let options = MySqlConnectOptions::from_str(&create_datasource_req.datasource_url)?;
    let host = options.get_host();
    let port = options.get_port();

    DataSourceDao::create(
        &pool,
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
pub async fn get_datasource_list(State(state): State<Pool<MySql>>) -> Result<Response, Infallible> {
    handle_response!(get_datasource_list_with_error(state).await)
}
async fn get_datasource_list_with_error(pool: Pool<MySql>) -> Result<String, anyhow::Error> {
    let redis_util = DataSourceDao::fetch_all_datasources(&pool).await?;
    let data = BaseResponse {
        response_code: 0,
        response_object: redis_util,
    };
    serde_json::to_string(&data).map_err(|e| anyhow!("{}", e))
}
