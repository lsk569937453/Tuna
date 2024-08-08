use redis::cluster::{ClusterClient, ClusterClientBuilder};
use redis::AsyncCommands;
use redsync::RedisInstance;
use redsync::Redsync;
use std::time::Duration;

pub async fn init_redis() -> Result<(ClusterClient, Redsync<RedisInstance>), anyhow::Error> {
    let nodes = vec![
        "redis://127.0.0.1:7000/",
        "redis://127.0.0.1:7001/",
        "redis://127.0.0.1:7002/",
    ];
    let mut instance = vec![];
    for item in nodes.iter() {
        instance.push(RedisInstance::new(item.clone())?);
    }
    info!("nodes is :{:?}", nodes);
    let client = ClusterClientBuilder::new(nodes.clone())
        .connection_timeout(Duration::from_secs(1))
        .build()?;
    let rl = Redsync::new(instance);
    Ok((client, rl))
}
