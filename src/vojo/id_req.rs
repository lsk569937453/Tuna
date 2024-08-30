use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct IdReq {
    pub id: i32,
}
