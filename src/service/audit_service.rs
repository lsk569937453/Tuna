use crate::dao::datasource_dao::DataSourceDao;
use crate::dao::task_dao::TaskDao;
use crate::handle_response;
use crate::vojo::audit_task_req;
use crate::vojo::audit_task_req::AuditTaskReq;
use crate::vojo::base_response::BaseResponse;
use crate::vojo::create_task_req::CreateTaskReq;
use axum::extract::State;
use axum::response::IntoResponse;
use axum::response::Response;
use axum::Json;
use sqlx::{MySql, Pool};
use std::convert::Infallible;
pub async fn create_audit_task(
    State(state): State<Pool<MySql>>,
    Json(data): Json<AuditTaskReq>,
) -> Result<Response, Infallible> {
    handle_response!(create_audit_task_with_error(state, data).await)
}
async fn create_audit_task_with_error(
    pool: Pool<MySql>,
    audit_task_req: AuditTaskReq,
) -> Result<String, anyhow::Error> {
    // let from_datasource =
    //     DataSourceDao::find_by_id(&pool, create_datasource_req.from_datasource_id).await?;
    // let to_datasource =
    //     DataSourceDao::find_by_id(&pool, create_datasource_req.to_datasource_id).await?;
    // TaskDao::create_task(
    //     &pool,
    //     &create_datasource_req,
    //     from_datasource.datasource_url,
    //     to_datasource.datasource_url,
    // )
    // .await?;
    let data = BaseResponse {
        response_code: 0,
        response_object: 0,
    };
    serde_json::to_string(&data).map_err(|e| anyhow!("{}", e))
}
