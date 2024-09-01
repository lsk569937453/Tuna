use crate::dao::datasource_dao::DataSourceDao;
use crate::dao::sync_task_dao::SyncTaskDao;
use crate::handle_response;
use crate::vojo::base_response::BaseResponse;
use crate::vojo::create_audit_task_req::CreateTaskReq;
use axum::extract::Path;
use axum::extract::State;
use axum::response::IntoResponse;
use axum::response::Response;
use axum::Json;
use sqlx::{MySql, Pool};
use std::convert::Infallible;

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
    let from_datasource =
        DataSourceDao::find_by_id(&pool, create_datasource_req.from_datasource_id).await?;
    let to_datasource =
        DataSourceDao::find_by_id(&pool, create_datasource_req.to_datasource_id).await?;
    SyncTaskDao::create_task(
        &pool,
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
pub async fn get_task_list(State(state): State<Pool<MySql>>) -> Result<Response, Infallible> {
    handle_response!(get_task_list_with_error(state).await)
}
async fn get_task_list_with_error(pool: Pool<MySql>) -> Result<String, anyhow::Error> {
    let res: Vec<SyncTaskDao> = SyncTaskDao::fetch_all_tasks(&pool).await?;
    let data = BaseResponse {
        response_code: 0,
        response_object: res,
    };
    serde_json::to_string(&data).map_err(|e| anyhow!("{}", e))
}
pub async fn delete_sync_task_by_id(
    State(state): State<Pool<MySql>>,
    Path(data): Path<i32>,
) -> Result<Response, Infallible> {
    handle_response!(delete_sync_task_by_id_with_error(state, data).await)
}
async fn delete_sync_task_by_id_with_error(
    pool: Pool<MySql>,
    id: i32,
) -> Result<String, anyhow::Error> {
    let redis_util = SyncTaskDao::delete_task(&pool, id).await?;
    let data = BaseResponse {
        response_code: 0,
        response_object: redis_util,
    };
    serde_json::to_string(&data).map_err(|e| anyhow!("{}", e))
}
