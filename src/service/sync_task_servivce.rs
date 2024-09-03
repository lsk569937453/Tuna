use crate::common::app_state::AppState;
use crate::common::common_constants::TASK_GID_KEY_TEMPLATE;
use crate::common::common_constants::TASK_INFO_KEY_TEMPLATE;
use crate::dao::datasource_dao::DataSourceDao;
use crate::dao::sync_task_dao::SyncTaskDao;
use crate::handle_response;
use crate::vojo::base_response::BaseResponse;
use crate::vojo::create_audit_task_req::CreateTaskReq;
use crate::vojo::sync_task_status_res::SyncTaskStatus;
use crate::vojo::sync_task_status_res::SyncTaskStatusRes;
use axum::extract::Path;
use axum::extract::State;
use axum::response::IntoResponse;
use axum::response::Response;
use axum::Json;
use redis::AsyncCommands;
use std::collections::HashMap;
use std::convert::Infallible;

pub async fn create_task(
    State(state): State<AppState>,
    Json(data): Json<CreateTaskReq>,
) -> Result<Response, Infallible> {
    handle_response!(create_task_with_error(state, data).await)
}
async fn create_task_with_error(
    app_state: AppState,
    create_datasource_req: CreateTaskReq,
) -> Result<String, anyhow::Error> {
    let from_datasource =
        DataSourceDao::find_by_id(&app_state.db_pool, create_datasource_req.from_datasource_id)
            .await?;
    let to_datasource =
        DataSourceDao::find_by_id(&app_state.db_pool, create_datasource_req.to_datasource_id)
            .await?;
    SyncTaskDao::create_task(
        &app_state.db_pool,
        &create_datasource_req,
        from_datasource.datasource_url,
        to_datasource.datasource_url,
    )
    .await?;
    let data = BaseResponse {
        response_code: 0,
        response_object: 0,
    };
    serde_json::to_string(&data).map_err(|e| anyhow!("{}", e))
}
pub async fn get_task_list(State(state): State<AppState>) -> Result<Response, Infallible> {
    handle_response!(get_task_list_with_error(state).await)
}
async fn get_task_list_with_error(app_state: AppState) -> Result<String, anyhow::Error> {
    let res: Vec<SyncTaskDao> = SyncTaskDao::fetch_all_tasks(&app_state.db_pool).await?;
    let data = BaseResponse {
        response_code: 0,
        response_object: res,
    };
    serde_json::to_string(&data).map_err(|e| anyhow!("{}", e))
}
pub async fn delete_sync_task_by_id(
    State(state): State<AppState>,
    Path(data): Path<i32>,
) -> Result<Response, Infallible> {
    handle_response!(delete_sync_task_by_id_with_error(state, data).await)
}
async fn delete_sync_task_by_id_with_error(
    app_state: AppState,
    id: i32,
) -> Result<String, anyhow::Error> {
    let redis_util = SyncTaskDao::delete_task(&app_state.db_pool, id).await?;
    let data = BaseResponse {
        response_code: 0,
        response_object: redis_util,
    };
    serde_json::to_string(&data).map_err(|e| anyhow!("{}", e))
}
pub async fn get_sync_task_status_by_id(
    State(state): State<AppState>,
    Path(data): Path<i32>,
) -> Result<Response, Infallible> {
    handle_response!(get_sync_task_status_by_id_with_error(state, data).await)
}
async fn get_sync_task_status_by_id_with_error(
    app_state: AppState,
    id: i32,
) -> Result<String, anyhow::Error> {
    let mut async_connection = app_state.redis_client.get_async_connection().await?;
    let redis_key = format!("{}{}", TASK_INFO_KEY_TEMPLATE, id);
    let redis_res: Option<String> = async_connection
        .get(redis_key)
        .await
        .map_err(|e| anyhow!("get_sync_task_status_by_id_with_error error:{:?}", e))?;
    let status = match redis_res {
        Some(r) => SyncTaskStatus::RUNNING { status: 1, ip: r },
        None => SyncTaskStatus::STOP { status: 1 },
    };
    let gtid_redis_key = format!("{}{}", TASK_GID_KEY_TEMPLATE, id);

    let h_res: HashMap<String, String> = async_connection.hgetall(gtid_redis_key).await?;
    let gtid_set = h_res
        .iter()
        .map(|(key, value)| format!("{}:{}", key, value))
        .collect::<Vec<String>>()
        .join(",");
    let sync_task_status_res = SyncTaskStatusRes { status, gtid_set };

    let data = BaseResponse {
        response_code: 0,
        response_object: sync_task_status_res,
    };
    serde_json::to_string(&data).map_err(|e| anyhow!("{}", e))
}
