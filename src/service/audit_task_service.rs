use crate::dao::audit_task_dao::AuditTaskDao;
use crate::dao::audit_task_result_dao::AuditTaskResultDao;
use crate::dao::sync_task_dao::SyncTaskDao;
use crate::handle_response;
use crate::vojo::audit_task_req::AuditTaskReq;
use crate::vojo::base_response::BaseResponse;
use crate::vojo::create_audit_task_req::TableMappingItem;
use crate::vojo::id_req::IdReq;
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
    SyncTaskDao::get_task(&pool, audit_task_req.task_id).await?;
    let id = AuditTaskDao::create_auit_task(&pool, audit_task_req.task_id).await?;
    let data = BaseResponse {
        response_code: 0,
        response_object: id,
    };
    serde_json::to_string(&data).map_err(|e| anyhow!("{}", e))
}
pub async fn get_audit_tasks(State(state): State<Pool<MySql>>) -> Result<Response, Infallible> {
    handle_response!(get_audit_tasks_with_error(state).await)
}
async fn get_audit_tasks_with_error(pool: Pool<MySql>) -> Result<String, anyhow::Error> {
    let audit_tasks = AuditTaskDao::fetch_all_audit_tasks(&pool).await?;
    let data = BaseResponse {
        response_code: 0,
        response_object: audit_tasks,
    };
    serde_json::to_string(&data).map_err(|e| anyhow!("{}", e))
}
pub async fn execute_audit_task(
    State(state): State<Pool<MySql>>,
    Json(data): Json<IdReq>,
) -> Result<Response, Infallible> {
    handle_response!(execute_audit_task_with_error(state, data).await)
}
async fn execute_audit_task_with_error(
    pool: Pool<MySql>,
    id_req: IdReq,
) -> Result<String, anyhow::Error> {
    let current_audit_task = AuditTaskDao::get_auit_task_by_id(&pool, id_req.id)
        .await?
        .ok_or(anyhow!("Can't find audit task by id {}", id_req.id))?;
    if current_audit_task.status == 1 {
        info!("audit task {} has been executed", id_req.id);
        let data = BaseResponse {
            response_code: 0,
            response_object: current_audit_task.status,
        };
        return serde_json::to_string(&data).map_err(|e| anyhow!("{}", e));
    }
    AuditTaskDao::update_auit_task_status(&pool, id_req.id, 1).await?;
    let task_id = id_req.id;
    let audit_task_id = current_audit_task.id;
    tokio::spawn(async move {
        if let Err(e) =
            async_execute_audit_task_with_error(pool.clone(), task_id, audit_task_id).await
        {
            error!(
                "async execute audit task {} error,error is {}",
                id_req.id, e
            );
        }
        if let Err(e) = AuditTaskDao::update_auit_task_status(&pool, id_req.id, 2).await {
            error!(
                "async execute audit task {} error,error is {}",
                id_req.id, e
            );
        }
    });

    let data = BaseResponse {
        response_code: 0,
        response_object: 0,
    };
    serde_json::to_string(&data).map_err(|e| anyhow!("{}", e))
}
async fn async_execute_audit_task_with_error(
    pool: Pool<MySql>,
    task_id: i32,
    audit_task_id: i32,
) -> Result<(), anyhow::Error> {
    info!("start execute audit task {}", task_id);
    let sync_task_dao = SyncTaskDao::get_task(&pool, task_id)
        .await?
        .ok_or(anyhow!("Can't find sync task by id {}", task_id))?;
    let table_mapping = sync_task_dao.table_mapping.clone();
    let table_mapping: HashMap<String, TableMappingItem> = serde_json::from_str(&table_mapping)?;
    for (_, table_mapping_item) in table_mapping.iter() {
        do_execute(
            pool.clone(),
            table_mapping_item.clone(),
            sync_task_dao.clone(),
            audit_task_id,
        )
        .await?;
    }
    info!("end execute audit task {}", task_id);
    Ok(())
}
async fn do_execute(
    pool: Pool<MySql>,
    table_mapping_item: TableMappingItem,
    sync_task_dao: SyncTaskDao,
    audit_task_id: i32,
) -> Result<(), anyhow::Error> {
    let mut from_mysql_connection =
        MySqlConnection::connect(&sync_task_dao.from_datasource_url).await?;
    let mut to_mysql_connection =
        MySqlConnection::connect(&sync_task_dao.to_datasource_url).await?;
    let left_compare = compare(
        &mut from_mysql_connection,
        table_mapping_item.from_primary_key.clone(),
        sync_task_dao.from_database_name.clone(),
        table_mapping_item.from_table_name.clone(),
        &mut to_mysql_connection,
        table_mapping_item.to_primary_key.clone(),
        sync_task_dao.to_database_name.clone(),
        table_mapping_item.to_table_name.clone(),
    )
    .await?;
    let right_compare = compare(
        &mut to_mysql_connection,
        table_mapping_item.to_primary_key,
        sync_task_dao.to_database_name,
        table_mapping_item.to_table_name.clone(),
        &mut from_mysql_connection,
        table_mapping_item.from_primary_key,
        sync_task_dao.from_database_name,
        table_mapping_item.from_table_name.clone(),
    )
    .await?;

    AuditTaskResultDao::create_task_result(
        &pool,
        audit_task_id,
        Some(left_compare),
        Some(right_compare),
    )
    .await?;
    Ok(())
}
async fn compare(
    from_mysql_connection: &mut MySqlConnection,
    from_primary_key: String,
    from_database_name: String,
    from_table_name: String,
    to_mysql_connection: &mut MySqlConnection,
    to_primary_key: String,

    to_database_name: String,
    to_table_name: String,
) -> Result<String, anyhow::Error> {
    let select_sql = format!("select * from {}.{}", from_database_name, from_table_name);
    let source_data = get_all_data(from_mysql_connection, select_sql, from_primary_key).await?;
    info!("source_data count:{}", source_data.len());
    let to_select_sql = format!(
        "select *from {}.{} where {}=?",
        to_database_name, to_table_name, to_primary_key
    );
    info!("to_select_sql:{}", to_select_sql);
    let mut result = vec![];
    for (key, value) in source_data.iter() {
        let data = get_one(to_mysql_connection, to_select_sql.clone(), key.clone()).await?;
        if let Some(data) = data {
            let bool = value.clone() == data;
            info!("source:{:?},dst:{:?},result:{}", value, data, bool);

            if !bool {
                result.push(key.clone());
            }
        } else {
            error!("no data,key is :{},value:{:?}", key, value);
            result.push(key.clone());
        }
    }
    let res = serde_json::to_string(&result)?;
    Ok(res)
}
async fn get_all_data(
    mysql_connection: &mut MySqlConnection,
    sql: String,
    primary_key: String,
) -> Result<HashMap<Value, LinkedList<Value>>, anyhow::Error> {
    let mut hash_map = HashMap::new();
    let results = sqlx::query(&sql).fetch_all(mysql_connection).await?;
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
    key: Value,
) -> Result<Option<LinkedList<Value>>, anyhow::Error> {
    let results = sqlx::query(&sql)
        .bind(key)
        .fetch_optional(mysql_connection)
        .await?;
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
    if raw_value.is_null() {
        return Value::Null;
    }
    let type_info = raw_value.type_info();
    let type_name = type_info.name();
    info!("type_name:,raw_value:{} ", type_name);
    match type_name {
        "REAL" | "FLOAT" | "NUMERIC" | "DECIMAL" | "FLOAT4" | "FLOAT8" | "DOUBLE" => {
            <f64 as Decode<sqlx::mysql::MySql>>::decode(raw_value)
                .unwrap_or(f64::NAN)
                .into()
        }
        "INT8" | "BIGINT" => <i64 as Decode<sqlx::mysql::MySql>>::decode(raw_value)
            .unwrap_or_default()
            .into(),
        "INT" | "INT4" | "INTEGER" | "TINYINT" => {
            <i32 as Decode<sqlx::mysql::MySql>>::decode(raw_value)
                .unwrap_or_default()
                .into()
        }
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
