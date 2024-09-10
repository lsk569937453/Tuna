use crate::binlog::binlog_poller::BinlogPoller;
use crate::common::app_state::AppState;
use crate::common::common_constants::TASK_INFO_KEY_TEMPLATE;
use crate::dao::sync_task_dao::SyncTaskDao;
use crate::record_error;
use redis::{cluster_async::ClusterConnection, AsyncCommands};

use std::time::Duration;
use tokio::time::interval;
#[instrument(name = "sync binlog", skip(cluster_connection, app_state))]
pub async fn sync_binlog_with_error(
    cluster_connection: &mut ClusterConnection,
    app_state: AppState,
    task_dao: SyncTaskDao,
) -> Result<(), anyhow::Error> {
    let duration = 5000;
    let mut interval = interval(Duration::from_millis(duration));

    let mut binlog_poller = BinlogPoller::start(
        task_dao.clone(),
        cluster_connection.clone(),
        app_state.clickhouse_client,
    )
    .await?;

    loop {
        let mut cloned_cluster_connection = cluster_connection.clone();
        tokio::select! {
            _ = interval.tick() => {
            record_error!(send_heartbeat_with_error(&mut cloned_cluster_connection,task_dao.clone()).await);
            },
            res=binlog_poller.poll() => {
                    res?;
            }
        }
    }
}
#[instrument(name = "send_heartbeat", skip(cluster_connection))]
async fn send_heartbeat_with_error(
    cluster_connection: &mut ClusterConnection,
    task_dao: SyncTaskDao,
) -> Result<(), anyhow::Error> {
    let task_info_key = format!("{}{}", TASK_INFO_KEY_TEMPLATE, task_dao.id);
    let _: () = cluster_connection
        .pexpire(task_info_key.clone(), 10000)
        .await
        .map_err(|e| anyhow!(e.to_string()))?;
    let current_ttl: i32 = cluster_connection
        .pttl(task_info_key.clone())
        .await
        .map_err(|e| anyhow!(e.to_string()))?;
    info!(
        "send_heartbeat success,task_id:{},current_ttl:{}ms",
        task_info_key, current_ttl
    );

    Ok(())
}
