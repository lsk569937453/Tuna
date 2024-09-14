use redis::cluster::{ClusterClient, ClusterClientBuilder};

use std::time::Duration;

use crate::config::tuna_config::RedisConfig;

pub async fn init_redis(redis_config: &RedisConfig) -> Result<ClusterClient, anyhow::Error> {
    // let nodes = vec![
    //     "redis://127.0.0.1:7000/",
    //     "redis://127.0.0.1:7001/",
    //     "redis://127.0.0.1:7002/",
    // ];
    let nodes = redis_config.urls.clone();

    info!("nodes is :{:?}", nodes);
    let client = ClusterClientBuilder::new(nodes.clone())
        .connection_timeout(Duration::from_secs(5))
        .response_timeout(Duration::from_secs(5))
        .build()?;
    //如果redis集群不可用，会报错
    let _ = client.get_async_connection().await?;

    Ok(client)
}
