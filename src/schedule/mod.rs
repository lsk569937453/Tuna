pub mod sync_binlog;
pub mod sync_redis;
#[macro_export]
macro_rules! record_error {
    ($result:expr) => {
        if let Err(e) = $result {
            error!("{}", e)
        }
    };
}
