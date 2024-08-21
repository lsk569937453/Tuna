use crate::dao::task_dao::TaskDao;
use crate::record_error;
use redis::{cluster_async::ClusterConnection, AsyncCommands};
use tokio::time::sleep;

use crate::binlog::binlog_poller::BinlogPoller;
use sqlx::MySql;
use sqlx::Pool;
use std::time::Duration;
use tokio::time::interval;
#[instrument(name = "sync binlog", skip(cluster_connection, pool))]
pub async fn sync_binlog_with_error(
    cluster_connection: &mut ClusterConnection,
    pool: Pool<MySql>,
    task_dao: TaskDao,
) -> Result<(), anyhow::Error> {
    let duration = 5000;
    let mut interval = interval(Duration::from_millis(duration));
    let mut binlog_poller =
        BinlogPoller::start(task_dao.clone(), cluster_connection.clone()).await?;
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
    Ok(())
}
#[instrument(skip(cluster_connection))]
async fn send_heartbeat_with_error(
    cluster_connection: &mut ClusterConnection,
    task_dao: TaskDao,
) -> Result<(), anyhow::Error> {
    let task_info_key = format!("tuna:task:{}", task_dao.id);
    cluster_connection
        .pexpire(task_info_key.clone(), 10000)
        .await?;
    let current_ttl: i32 = cluster_connection.pttl(task_info_key.clone()).await?;
    info!(
        "send_heartbeat success,task_id:{},current_ttl:{}ms",
        task_info_key, current_ttl
    );

    Ok(())
}
