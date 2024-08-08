use crate::common::common_constants::REDIS_TASK_INFO;
use crate::dao::task_dao::TaskDao;
use crate::record_error;
use redis::{cluster::ClusterClient, cluster_async::ClusterConnection, AsyncCommands};
use redsync::RedisInstance;
use redsync::Redsync;
use sqlx::MySql;
use sqlx::Pool;
use std::time::Duration;
use tokio::time::interval;

pub async fn sync_binlog_with_error(
    cluster_connection: &mut ClusterConnection,
    pool: Pool<MySql>,
    lock_manager: Redsync<RedisInstance>,
    task_id: i32,
) -> Result<(), anyhow::Error> {
    let duration = 5000;
    let mut interval = interval(Duration::from_millis(duration));

    loop {
        let mut cloned_cluster_connection = cluster_connection.clone();
        tokio::select! {
            _ = interval.tick() => {
            record_error!(send_heartbeat_with_error(&mut cloned_cluster_connection,task_id).await);
            },
        }
    }
    Ok(())
}

async fn send_heartbeat_with_error(
    cluster_connection: &mut ClusterConnection,
    task_id: i32,
) -> Result<(), anyhow::Error> {
    let task_info_key = format!("tuna:task:{}", task_id);
    cluster_connection
        .expire(task_info_key.clone(), 10000)
        .await?;
    info!("send_heartbeat success,task_id:{}", task_id);
    let current_ttl: i32 = cluster_connection.ttl(task_info_key).await?;
    info!("current_ttl:{}", current_ttl);

    Ok(())
}
