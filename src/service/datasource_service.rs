use crate::handle_response;
use crate::vojo::base_response::BaseResponse;
use crate::vojo::create_datasource_req;
use crate::vojo::create_datasource_req::CreateDatasourceReq;
use axum::extract::State;
use axum::http::header;
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

    sqlx::query(
        "INSERT INTO datasource (datasource_name, datasource_url, host, port) VALUES (?,?,?,?)",
    )
    .bind(create_datasource_req.datasource_name)
    .bind(create_datasource_req.datasource_url)
    .bind(host)
    .bind(port)
    .execute(&pool)
    .await?;
    let data = BaseResponse {
        response_code: 0,
        response_object: 0,
    };
    serde_json::to_string(&data).map_err(|e| anyhow!("{}", e))
}
