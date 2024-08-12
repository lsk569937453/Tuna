use std::collections::HashMap;

use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct CreateTaskReq {
    pub task_name: String,
    pub from_datasource_id: i32,
    pub to_datasource_id: i32,

    pub from_database_name: String,
    pub to_database_name: String,
    pub table_mapping: HashMap<String, String>,
}
