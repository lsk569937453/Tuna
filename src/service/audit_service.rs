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
use serde_json::Value;
use sqlx::mysql::MySqlValueRef;
use sqlx::Column;
use sqlx::Connection;
use sqlx::Decode;
use sqlx::MySqlConnection;
use sqlx::Row;
use sqlx::TypeInfo;
use sqlx::ValueRef;
use sqlx::{MySql, Pool};
use std::collections::HashMap;
use std::collections::LinkedList;
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
    let select_sql = format!(
        "select * from {}.{}",
        task_dao.from_database_name, task_dao.table_mapping
    );
    let from_mysql_connection = MySqlConnection::connect(&task_dao.from_datasource_url).await?;
    let mut to_mysql_connection = MySqlConnection::connect(&task_dao.to_datasource_url).await?;

    let source_data = get_all_data(
        from_mysql_connection,
        select_sql,
        audit_task_req.source_primary_key,
    )
    .await?;
    let to_select_sql = format!(
        "select *from {}.{} where {}=?",
        task_dao.to_database_name, task_dao.table_mapping, audit_task_req.destination_primary_key
    );
    for (key, value) in source_data.iter() {
        let data = get_one(&mut to_mysql_connection, to_select_sql.clone()).await?;
        if let Some(data) = data {
            let bool = value.clone() == data;
            info!("source:{:?},dst:{:?},result:{}", value, data, bool);
        } else {
            error!("no data");
        }
    }
    let data = BaseResponse {
        response_code: 0,
        response_object: 0,
    };
    serde_json::to_string(&data).map_err(|e| anyhow!("{}", e))
}
async fn get_all_data(
    mut mysql_connection: MySqlConnection,
    sql: String,
    primary_key: String,
) -> Result<HashMap<Value, LinkedList<Value>>, anyhow::Error> {
    let mut hash_map = HashMap::new();
    let results = sqlx::query(&sql).fetch_all(&mut mysql_connection).await?;
    for (_, iterate) in results.iter().enumerate() {
        let mut linked_list = LinkedList::new();
        let mut primary_value = Value::Null;
        for (index, column) in iterate.columns().iter().enumerate() {
            let raw_value = iterate.try_get_raw(index)?;
            let value = parse_value(raw_value).await;
            linked_list.push_back(value.clone());
            if column.name() == primary_key {
                primary_value = value;
            }
        }
        hash_map.insert(primary_value, linked_list);
    }
    Ok(hash_map)
}
async fn get_one(
    mysql_connection: &mut MySqlConnection,
    sql: String,
) -> Result<Option<LinkedList<Value>>, anyhow::Error> {
    let results = sqlx::query(&sql).fetch_optional(mysql_connection).await?;
    match results {
        None => Ok(None),
        Some(iterate) => {
            let mut linked_list = LinkedList::new();
            for (index, column) in iterate.columns().iter().enumerate() {
                let raw_value = iterate.try_get_raw(index)?;
                let value = parse_value(raw_value).await;
                linked_list.push_back(value.clone());
            }
            Ok(Some(linked_list))
        }
    }
}
async fn parse_value<'r>(raw_value: MySqlValueRef<'r>) -> Value {
    let type_info = raw_value.type_info();
    let type_name = type_info.name();
    match type_name {
        "REAL" | "FLOAT" | "NUMERIC" | "DECIMAL" | "FLOAT4" | "FLOAT8" | "DOUBLE" => {
            <f64 as Decode<sqlx::mysql::MySql>>::decode(raw_value)
                .unwrap_or(f64::NAN)
                .into()
        }
        "INT8" | "BIGINT" => <i64 as Decode<sqlx::mysql::MySql>>::decode(raw_value)
            .unwrap_or_default()
            .into(),
        "INT" | "INT4" | "INTEGER" => <i32 as Decode<sqlx::mysql::MySql>>::decode(raw_value)
            .unwrap_or_default()
            .into(),
        "INT2" | "SMALLINT" => <i16 as Decode<sqlx::mysql::MySql>>::decode(raw_value)
            .unwrap_or_default()
            .into(),
        "BOOL" | "BOOLEAN" => <bool as Decode<sqlx::mysql::MySql>>::decode(raw_value)
            .unwrap_or_default()
            .into(),
        "TIMESTAMP" | "DATE" => {
            <chrono::NaiveDate as Decode<sqlx::mysql::MySql>>::decode(raw_value)
                .as_ref()
                .map_or_else(std::string::ToString::to_string, ToString::to_string)
                .into()
        }

        "JSON" | "JSON[]" | "JSONB" | "JSONB[]" => {
            <Value as Decode<sqlx::mysql::MySql>>::decode(raw_value).unwrap_or_default()
        }
        // Deserialize as a string by default
        _ => <String as Decode<sqlx::mysql::MySql>>::decode(raw_value)
            .unwrap_or_default()
            .into(),
    }
}
