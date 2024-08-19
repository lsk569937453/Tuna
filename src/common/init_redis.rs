use redis::cluster::{ClusterClient, ClusterClientBuilder};

use std::time::Duration;

pub async fn init_redis() -> Result<ClusterClient, anyhow::Error> {
    let nodes = vec![
        "redis://127.0.0.1:7000/",
        "redis://127.0.0.1:7001/",
        "redis://127.0.0.1:7002/",
    ];

    info!("nodes is :{:?}", nodes);
    let client = ClusterClientBuilder::new(nodes.clone())
        .connection_timeout(Duration::from_secs(1))
        .build()?;
    //如果redis集群不可用，会报错
    let _ = client.get_async_connection().await?;

    Ok(client)
}
