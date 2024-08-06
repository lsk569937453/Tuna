use crate::dao::datasource_dao::DataSourceDao;
use crate::dao::task_dao::TaskDao;
use crate::handle_response;
use crate::vojo::base_response::BaseResponse;
use crate::vojo::create_datasource_req;
use crate::vojo::create_datasource_req::CreateDatasourceReq;
use crate::vojo::create_task_req::CreateTaskReq;
use crate::vojo::get_datasource_list_response::GetDatasourceListResponse;
use axum::extract::State;
use axum::http::header;
use axum::response::IntoResponse;
use axum::response::Response;
use axum::Json;
use sqlx::mysql::MySqlConnectOptions;
use sqlx::{MySql, Pool};
use std::convert::Infallible;
use std::str::FromStr;
pub async fn create_task(
    State(state): State<Pool<MySql>>,
    Json(data): Json<CreateTaskReq>,
) -> Result<Response, Infallible> {
    handle_response!(create_task_with_error(state, data).await)
}
async fn create_task_with_error(
    pool: Pool<MySql>,
    create_datasource_req: CreateTaskReq,
) -> Result<String, anyhow::Error> {
    TaskDao::create_task(&pool, &create_datasource_req).await?;
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
    let res: Vec<GetDatasourceListResponse> = DataSourceDao::fetch_all_datasources(&pool)
        .await?
        .into_iter()
        .map(|item| {
            let addr = format!("{}:{}", item.host, item.port);
            return GetDatasourceListResponse::new(item.datasource_name, addr, item.timestamp);
        })
        .collect();
    let data = BaseResponse {
        response_code: 0,
        response_object: res,
    };
    serde_json::to_string(&data).map_err(|e| anyhow!("{}", e))
}
