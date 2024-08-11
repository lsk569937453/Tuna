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
use sqlx::MySqlConnection;
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
    let task_dao = TaskDao::get_task(&pool, audit_task_req.task_id).await?;

    let data = BaseResponse {
        response_code: 0,
        response_object: 0,
    };
    serde_json::to_string(&data).map_err(|e| anyhow!("{}", e))
}
async fn get_all_data(
    mut mysql_connection: MySqlConnection,
    db_name: String,
    table_name: String,
) -> Result<(), anyhow::Error> {
    let select_sql = format!("select * from {}.{}", db_name, table_name);
    let results = sqlx::query(&select_sql)
        .fetch_all(&mut mysql_connection)
        .await?;
    for it in results.iter() {
        println!("{:?}", it.get::<i32>(0));
    }
    Ok(())
}
