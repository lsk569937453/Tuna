use crate::common::app_state::AppState;
use crate::common::common_constants::TASK_INFO_KEY_TEMPLATE;
use crate::common::common_constants::TASK_LOCK_KEY_TEMPLATE;
use crate::dao::sync_task_dao::SyncTaskDao;
use crate::record_error;
use crate::schedule::binlog_polling_task::BinlogPollingTask;
use crate::schedule::sync_binlog::sync_binlog_with_error;
use crate::util::redis_util::lock;
use crate::util::redis_util::unlock;
use local_ip_address::local_ip;
use redis::ExistenceCheck;
use redis::SetOptions;
use redis::Value;
use redis::{cluster_async::ClusterConnection, AsyncCommands};
use std::net::IpAddr;
use std::net::Ipv4Addr;

use std::time::Duration;
use tokio::time::interval;

pub async fn main_sync_redis_loop_with_error(app_state: AppState) -> Result<(), anyhow::Error> {
    let duration = 5000;
    let mut interval = interval(Duration::from_millis(duration));
    info!("start main_loop,interval is {}", duration);
    let mut cluster_connection = app_state.redis_client.get_async_connection().await?;
    loop {
        interval.tick().await;
        if let Err(e) = sync_task_ids(&mut cluster_connection, app_state.clone()).await {
            error!("main_sync_redis_loop_with_error error:{:?}", e);
            continue;
        }
    }
}

//每个任务都会去遍历所有的任务，然后去抢任务执行
async fn sync_task_ids(
    cluster_connection: &mut ClusterConnection,
    app_state: AppState,
) -> Result<(), anyhow::Error> {
    let tasks = SyncTaskDao::fetch_all_tasks(&app_state.db_pool).await?;
    for task in tasks {
        let cloned_pool = app_state.clone();
        let mut cloned_cluster_connection = cluster_connection.clone();
        let task_id = task.id;
        let task_info_key = format!("{}{}", TASK_INFO_KEY_TEMPLATE, task_id);
        let task_info_option: Option<String> = cluster_connection
            .clone()
            .get(task_info_key.clone())
            .await?;
        if task_info_option.is_none() {
            let task_lock = format!("{}{}", TASK_LOCK_KEY_TEMPLATE, task_id);
            info!(
                "try to get the task_lock,task_id:{},task_lock is {}",
                task_id, task_lock
            );

            let (lock_val, lock_result) = lock(
                &mut cluster_connection.clone(),
                task_lock.clone(),
                Duration::from_millis(3000),
            )
            .await?;
            if lock_result {
                info!(
                    "get the task_lock success,task_id:{},task_lock is {}",
                    task_id, task_lock
                );
                let binlog_polling_task = BinlogPollingTask::new(
                    app_state.clone(),
                    task_info_key,
                    task_lock,
                    lock_val,
                    task,
                );

                tokio::spawn(async move { binlog_polling_task.run().await });
            } else {
                info!(
                    "get the task_lock failed,task_id:{},task_lock is {}",
                    task_id, task_lock
                );
                continue;
            }
        } else {
            info!(
                "Taskid {} is in the redis,maybe other thread has run with it.",
                task_id
            );
        }
    }
    Ok(())
}
