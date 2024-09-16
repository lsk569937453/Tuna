use crate::dao::sync_task_dao::SyncTaskDao;
use crate::schedule::sync_binlog::sync_binlog_with_error;
use crate::util::redis_util::unlock;
use crate::AppState;
use local_ip_address::local_ip;
use redis::cluster_async::ClusterConnection;
use redis::AsyncCommands;
use redis::ExistenceCheck;
use redis::SetOptions;
use redis::Value;
use std::net::IpAddr;
use std::net::Ipv4Addr;
use uuid::Uuid;
#[derive(Clone)]
pub struct BinlogPollingTask {
    pub sync_task_id: u32,
    pub sync_task_uuid: u128,
    pub polling_interval_ms: u64,
    pub task_info_key: String,
    pub task_lock: String,
    pub lock_val: String,
    pub sync_task_dao: SyncTaskDao,
    pub app_state: AppState,
}
impl BinlogPollingTask {
    pub fn new(
        app_state: AppState,
        task_info_key: String,
        task_lock: String,
        lock_val: String,
        sync_task_dao: SyncTaskDao,
    ) -> Self {
        BinlogPollingTask {
            app_state,
            sync_task_id: sync_task_dao.id as u32,
            sync_task_uuid: uuid::Uuid::new_v4().as_u128(),
            polling_interval_ms: 5000,
            task_info_key,
            task_lock,
            lock_val,
            sync_task_dao,
        }
    }
    pub async fn run(&self) {
        if let Err(e) = self.run_with_error().await {
            error!("run_with_error error: {:?}", e);
        }
    }
    pub async fn run_with_error(&self) -> Result<(), anyhow::Error> {
        let redis_cluster_client = self.app_state.redis_client.get_async_connection().await?;
        let mut cloned_cluster_connection = redis_cluster_client.clone();
        //获取到分布式锁的线程，先抢到锁，然后设置上task_info_key，再把锁释放掉
        //然后进入事件循环
        let client_ip = local_ip()
            .unwrap_or(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)))
            .to_string();

        let set_options = SetOptions::default()
            .conditional_set(ExistenceCheck::NX)
            .with_expiration(redis::SetExpiry::PX(10000));
        let operation_result: Result<Option<Value>, redis::RedisError> = cloned_cluster_connection
            .set_options(self.task_info_key.clone(), client_ip, set_options)
            .await;
        match operation_result {
            Ok(Some(Value::Okay)) => {
                unlock(
                    &mut cloned_cluster_connection,
                    self.task_lock.clone(),
                    self.lock_val.clone(),
                )
                .await;
                record_error!(
                    sync_binlog_with_error(
                        &mut cloned_cluster_connection,
                        self.app_state.clone(),
                        self.sync_task_dao.clone(),
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
        Ok(())
    }
}
