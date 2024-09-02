use crate::common::app_state;
use crate::common::app_state::AppState;
use crate::common::common_constants::TASK_INFO_KEY_TEMPLATE;
use crate::common::common_constants::TASK_LOCK_KEY_TEMPLATE;
use crate::dao::sync_task_dao::SyncTaskDao;
use crate::record_error;
use crate::schedule::sync_binlog::sync_binlog_with_error;
use crate::util::redis_util::lock;
use crate::util::redis_util::unlock;
use local_ip_address::local_ip;
use redis::ExistenceCheck;
use redis::SetOptions;
use redis::Value;
use redis::{cluster::ClusterClient, cluster_async::ClusterConnection, AsyncCommands};
use sqlx::MySql;
use sqlx::Pool;
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
    Ok(())
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
        let cloned_task = task.clone();
        let task_id = task.id;
        let task_info_key = format!("{}{}", TASK_INFO_KEY_TEMPLATE, task_id);
        let task_info_option: Option<String> = cluster_connection
            .clone()
            .get(task_info_key.clone())
            .await?;
        if task_info_option == None {
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

                tokio::spawn(async move {
                    //获取到分布式锁的线程，先抢到锁，然后设置上task_info_key，再把锁释放掉
                    //然后进入事件循环
                    let my_local_ip = local_ip().unwrap().to_string();

                    let set_options = SetOptions::default()
                        .conditional_set(ExistenceCheck::NX)
                        .with_expiration(redis::SetExpiry::PX(10000));
                    let operation_result: Result<Option<Value>, redis::RedisError> =
                        cloned_cluster_connection
                            .set_options(task_info_key, my_local_ip, set_options)
                            .await;
                    match operation_result {
                        Ok(Some(Value::Okay)) => {
                            unlock(&mut cloned_cluster_connection, task_lock, lock_val).await;
                            record_error!(
                                sync_binlog_with_error(
                                    &mut cloned_cluster_connection,
                                    cloned_pool,
                                    cloned_task,
                                )
                                .await
                            );
                        }
                        Err(e) => {
                            error!("set_options error: {:?}", e);
                        }
                        _ => {
                            info!("Lock fail");
                        }
                    }
                });
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
