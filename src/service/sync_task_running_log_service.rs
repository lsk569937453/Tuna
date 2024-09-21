use crate::common::app_state::AppState;

use crate::dao::sync_task_dao::SyncTaskDao;
use crate::dao::sync_task_running_log_dao::SyncTaskRunningLogsDao;
use crate::dao::sync_task_running_log_dao::SyncTaskSummaryByTaskIdDao;
use crate::handle_response;
use crate::vojo::base_response::BaseResponse;
use axum::extract::State;
use axum::response::IntoResponse;
use axum::response::Response;
use serde::Deserialize;
use serde::Serialize;
use std::convert::Infallible;
use time::OffsetDateTime;
#[derive(Debug, Serialize, Deserialize)]
pub struct SyncTaskSummaryByTaskIdDaoResponseItem {
    #[serde(flatten)]
    pub summary: SyncTaskSummaryByTaskIdDao,
    pub online: i32,
    pub duration_as_second: i64,
    pub sync_task_name: String,
}

pub async fn get_sync_task_running_logs_summary_group_by_sync_task_id(
    State(state): State<AppState>,
) -> Result<Response, Infallible> {
    handle_response!(
        get_sync_task_running_logs_summary_group_by_sync_task_id_with_error(state).await
    )
}
async fn get_sync_task_running_logs_summary_group_by_sync_task_id_with_error(
    app_state: AppState,
) -> Result<String, anyhow::Error> {
    let res: Vec<SyncTaskSummaryByTaskIdDao> =
        SyncTaskRunningLogsDao::get_sync_task_summary_by_task_id(app_state.clickhouse_client)
            .await?;
    let mut res_array = vec![];
    for summary in res.into_iter() {
        let sync_task_name = SyncTaskDao::get_task(&app_state.db_pool, summary.sync_task_id as i32)
            .await?
            .ok_or(anyhow!("Can not find task"))?
            .task_name;
        let lastest_duration = OffsetDateTime::now_utc() - summary.latest_timestamp;
        //超过60秒为离线
        let is_online = if lastest_duration.whole_seconds() > 60 {
            1
        } else {
            0
        };
        let duration = summary.latest_timestamp - summary.oldest_timestamp;
        let item = SyncTaskSummaryByTaskIdDaoResponseItem {
            summary,
            online: is_online,
            duration_as_second: duration.whole_seconds(),
            sync_task_name,
        };
        res_array.push(item);
    }
    // let res: Vec<SyncTaskSummaryByTaskIdDaoResponseItem> = res
    //     .into_iter()
    //     .map(|summary| {
    //         let lastest_duration = OffsetDateTime::now_utc() - summary.latest_timestamp;
    //         //超过60秒为离线
    //         let is_online = if lastest_duration.whole_seconds() > 60 {
    //             1
    //         } else {
    //             0
    //         };
    //         let duration = summary.latest_timestamp - summary.oldest_timestamp;
    //         SyncTaskSummaryByTaskIdDaoResponseItem {
    //             summary,
    //             online: is_online,
    //             duration_as_second: duration.whole_seconds(),
    //         }
    //     })
    //     .collect();
    let data = BaseResponse {
        response_code: 0,
        response_object: res_array,
    };
    serde_json::to_string(&data).map_err(|e| anyhow!("{}", e))
}
