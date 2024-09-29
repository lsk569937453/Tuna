use crate::binlog::binlog_poller::BinlogPoller;
use crate::common::common_constants::TASK_INFO_KEY_TEMPLATE;
use crate::dao::sql_logs_dao::SqlLogDao;
use crate::dao::sync_task_dao::SyncTaskDao;
use crate::dao::sync_task_running_log_dao::Loglevel;
use crate::dao::sync_task_running_log_dao::SyncTaskRunningLogsDao;
use crate::util::redis_util::unlock;
use crate::AppState;
use clickhouse::Client;
use local_ip_address::local_ip;
use redis::cluster_async::ClusterConnection;
use redis::AsyncCommands;
use redis::ExistenceCheck;
use redis::SetOptions;
use redis::Value;
use std::fmt::Debug;
use std::net::IpAddr;
use std::net::Ipv4Addr;
use std::time::Duration;
use std::time::Instant;
use tokio::time::interval;
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
impl Debug for BinlogPollingTask {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BinlogPollingTask")
            .field("sync_task_id", &self.sync_task_id)
            .field("sync_task_uuid", &self.sync_task_uuid)
            .field("polling_interval_ms", &self.polling_interval_ms)
            .field("task_info_key", &self.task_info_key)
            .field("task_lock", &self.task_lock)
            .field("lock_val", &self.lock_val)
            .field("sync_task_dao", &self.sync_task_dao)
            .finish()
    }
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
    #[instrument(name = "sync binlog", skip(self),fields(sync_task_id = %self.sync_task_id))]

    pub async fn run(&self) {
        if let Err(e) = self.run_with_error().await {
            error!("run_with_error error: {:?}", e);
            if let Err(e) = self
                .insert_sync_task_running_logs(Loglevel::Error, e.to_string())
                .await
            {
                error!("insert_sync_task_running_logs error: {:?}", e);
            }
        }
    }
    pub async fn run_with_error(&self) -> Result<(), anyhow::Error> {
        self.insert_sync_task_running_logs(Loglevel::Info, "Start running task.".to_string())
            .await?;
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
                    self.sync_binlog_with_error(&mut cloned_cluster_connection,)
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
    async fn insert_sync_task_running_logs(
        &self,
        log_level: Loglevel,
        message: String,
    ) -> Result<(), anyhow::Error> {
        let sync =
            SyncTaskRunningLogsDao::new(self.sync_task_uuid, log_level, message, self.sync_task_id);
        SyncTaskRunningLogsDao::insert_one(self.app_state.clickhouse_client.clone(), sync).await?;
        Ok(())
    }
    pub async fn sync_binlog_with_error(
        &self,
        redis_cluster_client: &mut ClusterConnection,
    ) -> Result<(), anyhow::Error> {
        let duration = self.polling_interval_ms;
        let mut interval = interval(Duration::from_millis(duration));

        let mut binlog_poller =
            BinlogPoller::start(self.sync_task_dao.clone(), redis_cluster_client.clone()).await?;
        let clickhouse_client = self.app_state.clickhouse_client.clone();
        let mut sql_logs = vec![];
        loop {
            let mut cloned_cluster_connection = redis_cluster_client.clone();
            tokio::select! {
                _ = interval.tick() => {
                    record_error!(self.send_heartbeat_with_error(&mut cloned_cluster_connection,self.sync_task_dao.clone(),&mut  sql_logs,clickhouse_client.clone()).await);
                },
                res=binlog_poller.poll() => {
                        let mut current_sql_logs=res?;
                        sql_logs.append(&mut current_sql_logs);
                }
            }
        }
    }

    async fn send_heartbeat_with_error(
        &self,
        cluster_connection: &mut ClusterConnection,
        task_dao: SyncTaskDao,
        sql_logs: &mut Vec<SqlLogDao>,
        clickhouse_client: Client,
    ) -> Result<(), anyhow::Error> {
        self.insert_sync_task_running_logs(Loglevel::Info, "Send heartbeat.".to_string())
            .await?;
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
}
