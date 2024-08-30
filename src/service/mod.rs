pub mod audit_task_result_service;
pub mod audit_task_service;
pub mod database_service;
pub mod datasource_service;
pub mod table_service;
pub mod task_servivce;
#[macro_export]
macro_rules! handle_response {
    ($result:expr) => {
        match $result {
            Ok(r) => Ok((
                axum::http::StatusCode::OK,
                [(axum::http::header::CONTENT_TYPE, "application/json")],
                r,
            )
                .into_response()),
            Err(e) => {
                error!("{}", e);
                Ok((axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response())
            }
        }
    };
}
