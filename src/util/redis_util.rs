use rand::thread_rng;
use rand::RngCore;
use redis::ExistenceCheck;
use redis::RedisResult;
use redis::Value;
use redis::{cluster_async::ClusterConnection, AsyncCommands, SetOptions};
use std::io;
use std::time::Duration;
use uuid::Uuid;
const UNLOCK_SCRIPT: &str = r#"
if redis.call("GET", KEYS[1]) == ARGV[1] then
  return redis.call("DEL", KEYS[1])
else
  return 0
end
"#;
pub async fn lock(
    cluster_connection: &mut ClusterConnection,
    key: String,
    ttl: Duration,
) -> Result<(String, bool), anyhow::Error> {
    let val = get_unique_lock_id();
    let ttl = ttl.as_millis();
    let set_options = SetOptions::default()
        .conditional_set(ExistenceCheck::NX)
        .with_expiration(redis::SetExpiry::PX(ttl as u64));
    let result: Option<Value> = cluster_connection
        .set_options(key.clone(), val.clone(), set_options)
        .await?;
    let res = match result {
        Some(Value::Okay) => true,
        _ => false,
    };

    info!("lock key is {},value is {}，res:{},", key, val, res);

    Ok((val, res))
}
pub fn get_unique_lock_id() -> String {
    let uuid = Uuid::new_v4().to_string();
    uuid
}
pub async fn unlock(
    cluster_connection: &mut ClusterConnection,
    key: String,
    value: String,
) -> bool {
    let script = redis::Script::new(UNLOCK_SCRIPT);
    let result: RedisResult<i32> = script
        .key(key.clone())
        .arg(value.clone())
        .invoke_async(cluster_connection)
        .await;
    let res = match result {
        Ok(val) => val == 1,
        Err(_) => false,
    };
    info!("unlock key is {},value is {}，res:{},", key, value, res);
    res
}
