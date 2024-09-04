use serde::Serialize;

#[derive(Serialize, Clone)]
pub struct SyncTaskStatusRes {
    pub status: SyncTaskStatus,
    pub gtid_set: String,
}
// status: 0:running 1:stop
#[derive(Serialize, Clone)]
pub enum SyncTaskStatus {
    Running { status: u8, ip: String },
    Stop { status: u8 },
}
