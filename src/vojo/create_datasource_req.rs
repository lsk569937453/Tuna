use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct CreateDatasourceReq {
    pub datasource_name: String,
    pub datasource_url: String,
}
