use crate::common::app_state::AppState;
use crate::common::common_constants::TASK_INFO_KEY_TEMPLATE;
use crate::dao::sync_task_dao::SyncTaskDao;
use crate::record_error;
use crate::{binlog::binlog_poller::BinlogPoller, dao::sql_logs_dao::SqlLogDao};
use clickhouse::Client;
use redis::{cluster_async::ClusterConnection, AsyncCommands};

use std::time::Duration;
use tokio::time::{interval, Instant};
#[instrument(name = "sync binlog", skip(cluster_connection, app_state))]
pub async fn sync_binlog_with_error(
    cluster_connection: &mut ClusterConnection,
    app_state: AppState,
    task_dao: SyncTaskDao,
) -> Result<(), anyhow::Error> {
    let duration = 5000;
    let mut interval = interval(Duration::from_millis(duration));

    let mut binlog_poller =
        BinlogPoller::start(task_dao.clone(), cluster_connection.clone()).await?;
    let clickhouse_client = app_state.clickhouse_client;
    let mut sql_logs = vec![];
    loop {
        let mut cloned_cluster_connection = cluster_connection.clone();
        tokio::select! {
            _ = interval.tick() => {
                record_error!(send_heartbeat_with_error(&mut cloned_cluster_connection,task_dao.clone(),&mut  sql_logs,clickhouse_client.clone()).await);
            },
            res=binlog_poller.poll() => {
                    let mut current_sql_logs=res?;
                    sql_logs.append(&mut current_sql_logs);
            }
        }
    }
}

async fn send_heartbeat_with_error(
    cluster_connection: &mut ClusterConnection,
    task_dao: SyncTaskDao,
    sql_logs: &mut Vec<SqlLogDao>,
    clickhouse_client: Client,
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
    if !sql_logs.is_empty() {
        let now = Instant::now();
        let mut inserter = clickhouse_client.insert("sql_logs")?;
        for sql_log in sql_logs.iter() {
            inserter.write(sql_log).await?;
        }
        inserter.end().await?;
        let elapsed = now.elapsed().as_millis();
        info!(
            "insert clickhouse success,task count:{},elapsed:{}ms",
            sql_logs.len(),
            elapsed
        );
        sql_logs.clear();
    }

    info!(
        "send_heartbeat success,task_id:{},current_ttl:{}ms",
        task_info_key, current_ttl
    );

    Ok(())
}
