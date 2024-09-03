use serde::Serialize;

#[derive(Serialize, Clone)]
pub struct SyncTaskStatusRes {
    pub status: SyncTaskStatus,
    pub gtid_set: String,
}
// status: 0:running 1:stop
#[derive(Serialize, Clone)]
pub enum SyncTaskStatus {
    RUNNING { status: u8, ip: String },
    STOP { status: u8 },
}
