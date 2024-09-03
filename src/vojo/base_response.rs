use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize)]
pub struct BaseResponse<T>
where
    T: 'static,
{
    #[serde(rename = "resCode")]
    pub response_code: i32,
    #[serde(rename = "message")]
    pub response_object: T,
}
