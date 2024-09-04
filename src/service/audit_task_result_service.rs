use crate::common::app_state::AppState;
use crate::dao::audit_task_result_clickhouse_dao::AuditTaskResultClickhouseDao;
use crate::handle_response;
use crate::vojo::base_response::BaseResponse;
use axum::extract::State;
use axum::response::IntoResponse;
use axum::response::Response;
use std::convert::Infallible;

pub async fn get_audit_tasks_result(State(state): State<AppState>) -> Result<Response, Infallible> {
    handle_response!(get_audit_tasks_result_with_error(state).await)
}
async fn get_audit_tasks_result_with_error(app_state: AppState) -> Result<String, anyhow::Error> {
    let audit_tasks =
        AuditTaskResultClickhouseDao::get_audit_tasks_result_list(app_state.clickhouse_client)
            .await?;
    let data = BaseResponse {
        response_code: 0,
        response_object: audit_tasks,
    };
    serde_json::to_string(&data).map_err(|e| anyhow!("{}", e))
}
