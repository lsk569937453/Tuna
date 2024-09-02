use redis::cluster::ClusterClient;
use sqlx::MySql;
use sqlx::Pool;
#[derive(Clone)]
pub struct AppState {
    pub db_pool: Pool<MySql>,
    pub redis_client: ClusterClient,
}
