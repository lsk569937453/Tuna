use std::collections::HashMap;

use mysql_async::binlog::jsonb::Array;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Clone)]
pub struct CreateTaskReq {
    pub task_name: String,
    pub from_datasource_id: i32,
    pub to_datasource_id: i32,

    pub from_database_name: String,
    pub to_database_name: String,
    pub table_mapping: HashMap<String, TableMappingItem>,
}
#[derive(Deserialize, Clone, Serialize)]
pub struct TableMappingItem {
    pub from_table_name: String,
    pub to_table_name: String,
    pub from_primary_key: String,
    pub to_primary_key: String,
}
