use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct TableMapEventEntity {
    pub database_name: String,
    pub table_name: String,
}
