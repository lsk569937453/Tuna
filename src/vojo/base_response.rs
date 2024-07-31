use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize)]
pub struct BaseResponse<T>
where
    T: 'static,
{
    pub response_code: i32,
    pub response_object: T,
}

pub fn to_string<T>(response_code: i32, response_object: T) -> Result<String, anyhow::Error>
where
    T: Serialize + 'static,
{
    let base_response = BaseResponse {
        response_code,
        response_object,
    };
    serde_json::to_string(&base_response).map_err(|e| anyhow!("{}", e))
}
