use crate::common::app_state::AppState;
use crate::dao::audit_task_result_clickhouse_dao::AuditTaskResultClickhouseDao;
use crate::dao::sql_logs_dao::SqlLogDao;
use crate::dao::sync_task_dao::SyncTaskDao;
use crate::handle_response;
use crate::vojo::base_response::BaseResponse;
use crate::vojo::get_audit_task_result_by_audit_task_id_res::AuditTaskResultResponse;
use crate::vojo::logs_per_day_groupby_sync_task_id::LogsPerDayGroupbySyncTaskId;
use crate::vojo::logs_per_day_groupby_sync_task_id::LogsPerDayGroupbySyncTaskIdItem;
use crate::vojo::logs_per_minute_groupby_sync_task_id::LogsPerminuteGroupbySyncTaskIdRes;
use crate::vojo::logs_per_minute_groupby_sync_task_id::LogsPerminuteGroupbySyncTaskIdResItem;

use axum::extract::Path;
use axum::extract::State;
use axum::response::IntoResponse;
use axum::response::Response;
use std::collections::HashMap;
use std::collections::HashSet;
use std::convert::Infallible;
pub async fn get_sql_logs_per_minute(
    State(state): State<AppState>,
) -> Result<Response, Infallible> {
    handle_response!(get_sql_logs_per_minute_with_error(state,).await)
}
async fn get_sql_logs_per_minute_with_error(app_state: AppState) -> Result<String, anyhow::Error> {
    let logs = SqlLogDao::get_logs_per_minute(app_state.clickhouse_client).await?;
    let data = BaseResponse {
        response_code: 0,
        response_object: logs,
    };
    serde_json::to_string(&data).map_err(|e| anyhow!("{}", e))
}
pub async fn get_sql_logs_per_day(State(state): State<AppState>) -> Result<Response, Infallible> {
    handle_response!(get_sql_logs_per_day_with_error(state,).await)
}
async fn get_sql_logs_per_day_with_error(app_state: AppState) -> Result<String, anyhow::Error> {
    let logs = SqlLogDao::get_logs_per_day(app_state.clickhouse_client).await?;
    let data = BaseResponse {
        response_code: 0,
        response_object: logs,
    };
    serde_json::to_string(&data).map_err(|e| anyhow!("{}", e))
}
pub async fn get_sql_logs_per_minute_groupby_sync_task_id(
    State(state): State<AppState>,
) -> Result<Response, Infallible> {
    handle_response!(get_sql_logs_per_minute_groupby_sync_task_id_with_error(state,).await)
}
async fn get_sql_logs_per_minute_groupby_sync_task_id_with_error(
    app_state: AppState,
) -> Result<String, anyhow::Error> {
    let logs =
        SqlLogDao::get_logs_per_minute_groupby_sync_task_id(app_state.clickhouse_client).await?;
    let mut map = HashMap::new();
    let mut sync_task_id_set: HashSet<u32> = HashSet::new();
    for log in logs {
        let sync_task_map = map.entry(log.minute).or_insert_with(HashMap::new);
        sync_task_map.insert(log.sync_task_id, log.total_logs);
        sync_task_id_set.insert(log.sync_task_id);
    }
    let all_minutes: Vec<String> = {
        let mut minutes = map.keys().cloned().collect::<Vec<String>>();
        minutes.sort();
        minutes
    };
    let mut result = vec![];
    for sync_task_id in sync_task_id_set {
        let mut total_logs = vec![];
        for minute in &all_minutes {
            let minute_map = map.get(minute).ok_or(anyhow!("Map get error, not found"))?;
            total_logs.push(*minute_map.get(&sync_task_id).unwrap_or(&0));
        }
        let task_name = SyncTaskDao::get_task(&app_state.db_pool, sync_task_id as i32)
            .await?
            .ok_or(anyhow!("Can not find task"))?
            .task_name;
        let ss = LogsPerminuteGroupbySyncTaskIdResItem::new(task_name, total_logs);
        result.push(ss);
    }
    let data = LogsPerminuteGroupbySyncTaskIdRes::new(all_minutes, result);
    let data = BaseResponse {
        response_code: 0,
        response_object: data,
    };
    serde_json::to_string(&data).map_err(|e| anyhow!("{}", e))
}
pub async fn get_sql_logs_per_day_groupby_sync_task_id(
    State(state): State<AppState>,
) -> Result<Response, Infallible> {
    handle_response!(get_sql_logs_per_day_groupby_sync_task_id_with_error(state,).await)
}
async fn get_sql_logs_per_day_groupby_sync_task_id_with_error(
    app_state: AppState,
) -> Result<String, anyhow::Error> {
    let logs =
        SqlLogDao::get_logs_per_day_groupby_sync_task_id(app_state.clickhouse_client).await?;
    info!(
        "get_sql_logs_per_day_groupby_sync_task_id_with_error: {:?}",
        logs
    );
    let mut map = HashMap::new();

    for log in logs {
        map.entry(log.sync_task_id)
            .or_insert_with(Vec::new)
            .push(log);
    }
    let mut result = vec![];
    for (key, value) in map {
        let task_name = SyncTaskDao::get_task(&app_state.db_pool, key as i32)
            .await?
            .ok_or(anyhow!("Can not find task"))?
            .task_name;
        let list = value
            .into_iter()
            .map(|x| LogsPerDayGroupbySyncTaskIdItem::new(x.day, x.total_logs))
            .collect();
        let logs_perminute_groupby_sync_task_id = LogsPerDayGroupbySyncTaskId {
            sync_task_name: task_name,
            list,
        };
        result.push(logs_perminute_groupby_sync_task_id);
    }
    let data = BaseResponse {
        response_code: 0,
        response_object: result,
    };
    serde_json::to_string(&data).map_err(|e| anyhow!("{}", e))
}
