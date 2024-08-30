use crate::dao::audit_task_result_dao::AuditTaskResultDao;
use crate::handle_response;
use crate::vojo::base_response::BaseResponse;
use axum::extract::State;
use axum::response::IntoResponse;
use axum::response::Response;
use sqlx::{MySql, Pool};
use std::convert::Infallible;

pub async fn get_audit_tasks_result(
    State(state): State<Pool<MySql>>,
) -> Result<Response, Infallible> {
    handle_response!(get_audit_tasks_result_with_error(state).await)
}
async fn get_audit_tasks_result_with_error(pool: Pool<MySql>) -> Result<String, anyhow::Error> {
    let audit_tasks = AuditTaskResultDao::fetch_all_audit_tasks_result(&pool).await?;
    let data = BaseResponse {
        response_code: 0,
        response_object: audit_tasks,
    };
    serde_json::to_string(&data).map_err(|e| anyhow!("{}", e))
}
