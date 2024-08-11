use std::collections::HashMap;

use anyhow::anyhow;
use futures::StreamExt;
use moka::future::Cache;
use mysql_async::binlog::events::EventData;
use mysql_async::binlog::events::TableMapEvent;
use mysql_async::binlog::events::{RowsEventData, RowsEventRows};
use mysql_async::binlog::value::BinlogValue;
use mysql_async::BinlogStream;
use mysql_async::{Conn, Sid};
use mysql_async::{Opts, Value};
use redis::cluster_async::ClusterConnection;
use redis::AsyncCommands;
use sqlparser::ast::Statement;
use sqlparser::dialect::GenericDialect;
use sqlparser::parser::Parser;
use sqlx::mysql::MySqlConnection;
use sqlx::Connection;
use sqlx::Row;

use crate::dao::task_dao::TaskDao;

pub struct BinlogPoller {
    cluster_connection: ClusterConnection,
    task_dao: TaskDao,
    current_gtid_set: Option<String>,
    binlog_stream: BinlogStream,
    current_db_name: String,
    current_binlog_name: String,
    current_binlog_position: u32,
    current_table_map_event: Option<TableMapEvent<'static>>,
    column_list: Option<Vec<String>>,
    cache: Cache<String, Vec<String>>,
    source_mysql_connection: MySqlConnection,
    destination_mysql_connection: MySqlConnection,
}
//impl debug for me
impl std::fmt::Debug for BinlogPoller {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BinlogPoller")
            .field("gtid_set", &self.current_gtid_set)
            .field("current_db_name", &self.current_db_name)
            .field("current_binlog_name", &self.current_binlog_name)
            .field("current_binlog_position", &self.current_binlog_position)
            .finish()
    }
}
impl BinlogPoller {
    async fn get_gtid_set(
        mut cluster_connection: ClusterConnection,
        task_dao: TaskDao,
    ) -> Result<Option<Vec<String>>, anyhow::Error> {
        let gtid_set_key = format!("tuna:task:{}:gtid_set", task_dao.id);
        let hash_map: HashMap<String, String> = cluster_connection.hgetall(gtid_set_key).await?;
        if hash_map.is_empty() {
            return Ok(None);
        }
        let res = hash_map
            .iter()
            .map(|(k, v)| format!("{}:{}", k, v))
            .collect::<Vec<String>>();
        Ok(Some(res))
    }
    pub async fn start(
        task_dao: TaskDao,
        cluster_connection: ClusterConnection,
    ) -> Result<Self, anyhow::Error> {
        let from_datasource_url = task_dao.clone().from_datasource_url;
        let mysql = Conn::new(Opts::from_url(&from_datasource_url)?).await?;
        let mut sids = vec![];
        let get_gtid_option =
            BinlogPoller::get_gtid_set(cluster_connection.clone(), task_dao.clone()).await?;
        if let Some(gtid_list) = get_gtid_option {
            for gtid in gtid_list {
                sids.push(gtid.parse::<Sid>()?);
            }
        }
        info!("Pull binlog from gtid_set: {:?}", sids);
        let stream = mysql
            .get_binlog_stream(
                mysql_async::BinlogStreamRequest::new(11)
                    .with_gtid()
                    .with_gtid_set(sids),
            )
            .await?;
        let source =
            BinlogPoller::create_mysql_connection(task_dao.clone().from_datasource_url).await?;
        let destination =
            BinlogPoller::create_mysql_connection(task_dao.clone().to_datasource_url).await?;
        let sel = Self {
            cluster_connection: cluster_connection,
            task_dao: task_dao,
            current_gtid_set: None,
            binlog_stream: stream,
            current_db_name: String::new(),
            current_binlog_name: String::new(),
            current_binlog_position: 0,
            current_table_map_event: None,
            column_list: None,
            cache: Cache::new(1000),
            source_mysql_connection: source,
            destination_mysql_connection: destination,
        };
        Ok(sel)
    }
    pub async fn create_mysql_connection(
        datasource_url: String,
    ) -> Result<MySqlConnection, anyhow::Error> {
        let conn = MySqlConnection::connect(&datasource_url).await?;
        Ok(conn)
    }
    #[instrument]
    pub async fn poll(&mut self) -> Result<(), anyhow::Error> {
        if let Some(Ok(event)) = self.binlog_stream.next().await {
            self.current_binlog_position = event.header().log_pos();
            let event_cloned = event.clone();
            let option_event_data = event_cloned.read_data()?;
            let event_data = option_event_data.ok_or(anyhow!("Read data error"))?;

            let sql = match event_data {
                EventData::TableMapEvent(table_map_event) => {
                    let db_name = table_map_event.database_name();
                    let table_name = table_map_event.table_name();
                    let key = format!("{}{}", db_name, table_name);
                    if db_name != self.task_dao.source_database_name {
                        return Ok(());
                    }
                    let s = self.cache.get(&key).await;

                    //可能查询很多次
                    let current_column_list = if s.is_none() {
                        let v = self
                            .parse_colomns(
                                db_name.to_string().clone(),
                                table_name.to_string().clone(),
                            )
                            .await?;
                        self.cache
                            .insert(db_name.to_string().clone(), v.clone())
                            .await;
                        v
                    } else {
                        s.unwrap()
                    };
                    self.column_list = Some(current_column_list);
                    self.current_table_map_event = Some(table_map_event.into_owned());
                    None
                }
                EventData::RowsEvent(rows_event_data) => {
                    if !should_save(self.current_table_map_event.clone()) {
                        return Ok(());
                    }
                    let table_map_eventt = self.current_table_map_event.clone();
                    let data = table_map_eventt.ok_or(anyhow!(""))?;
                    let column_list = self.column_list.clone().ok_or(anyhow!(""))?;
                    let sql =
                        parse_sql_with_error(rows_event_data, data.clone(), column_list).await?;
                    Some(sql)
                }
                EventData::QueryEvent(query_event) => {
                    let sql = String::from_utf8_lossy(query_event.query_raw());
                    let dialect = GenericDialect {}; // or AnsiDialect, or your own dialect ...
                    let ast_list = Parser::parse_sql(&dialect, sql.as_ref())?;
                    let first_ast = ast_list.first().ok_or(anyhow!("parse_sql error"))?;
                    match first_ast {
                        Statement::StartTransaction {
                            modes,
                            begin,
                            modifier,
                        } => {
                            info!("QueryEvent:{}", sql);
                            Some("BEGIN".to_string())
                        }
                        _ => None,
                    }
                }
                EventData::RowsQueryEvent(rows_query_event) => {
                    info!("{:?}", rows_query_event);
                    None
                }
                EventData::TransactionPayloadEvent(transaction_payload_event) => {
                    info!("{:?}", transaction_payload_event);
                    None
                }

                EventData::GtidEvent(gtid_event) => {
                    let gtid = uuid::Uuid::from_bytes(gtid_event.sid());
                    info!(
                        "{}-{}-gtid:=============================={}:{}",
                        self.current_binlog_name,
                        self.current_binlog_position,
                        gtid.to_string(),
                        gtid_event.gno()
                    );
                    let gtid_sql = format!(
                        r#"set gtid_next='{}:{}';"#,
                        gtid.to_string(),
                        gtid_event.gno()
                    );
                    Some(gtid_sql)
                    //set gtid_next='UUID:5'
                }
                EventData::XidEvent(_) => {
                    info!(
                        "{}-{}-COMMIT;",
                        self.current_binlog_name, self.current_binlog_position,
                    );
                    Some("COMMIT".to_string())
                }
                EventData::RotateEvent(rotate_event) => {
                    self.current_binlog_name = rotate_event.name().to_string();
                    info!("rotate_event>>>{:?}", rotate_event);
                    None
                }
                EventData::HeartbeatEvent => None,
                EventData::FormatDescriptionEvent(e) => None,
                _ => {
                    info!("other>>>>>{:?}", event_data);
                    None
                }
            };
            if let Some(sql) = sql {
                self.handle_sql(sql).await;
            }
        }
        Ok(())
    }
    async fn parse_colomns(
        &mut self,
        database: String,
        table_name: String,
    ) -> Result<Vec<String>, anyhow::Error> {
        let  rows=sqlx::query("SELECT COLUMN_NAME FROM INFORMATION_SCHEMA.COLUMNS WHERE TABLE_SCHEMA = ? AND TABLE_NAME = ? ORDER BY ORDINAL_POSITION")
    .bind(database)
    .bind(table_name)
    .fetch_all(&mut self.source_mysql_connection)
    .await?;
        let mut res = vec![];
        for it in rows.iter() {
            let item: String = it.get(0);
            res.push(item);
        }
        Ok(res)
    }
    async fn handle_sql(&self, sql: String) {
        info!("current :sql:{}", sql);
    }
}
fn should_save(current_table_map_event: Option<TableMapEvent>) -> bool {
    if let Some(current_map_event) = current_table_map_event.clone() {
        let db_name = current_map_event.database_name();
        if db_name == "mydb" {
            return true;
        } else {
            return false;
        }
    } else {
        return false;
    }
}
async fn parse_sql_with_error(
    rows_event_data: RowsEventData<'_>,
    table_map_event: TableMapEvent<'_>,
    column_list: Vec<String>,
) -> Result<String, anyhow::Error> {
    let db_name = table_map_event.database_name().to_string();
    let table_name = table_map_event.table_name().to_string();
    match rows_event_data {
        RowsEventData::WriteRowsEvent(write_rows_event) => {
            let rows_event = write_rows_event.rows(&table_map_event);
            let insert_sql = parse_insert_sql(rows_event, db_name, table_name, column_list).await?;
            Ok(insert_sql)
        }
        RowsEventData::UpdateRowsEvent(update_rows_event) => {
            let rows_event = update_rows_event.rows(&table_map_event);
            let update_sql = parse_update_sql(rows_event, db_name, table_name, column_list).await?;
            Ok(update_sql)
        }
        RowsEventData::DeleteRowsEvent(delete_rows_event) => {
            let rows_event = delete_rows_event.rows(&table_map_event);
            let delete_sql = parse_delete_sql(rows_event, db_name, table_name, column_list).await?;
            Ok(delete_sql)
        }
        _ => Err(anyhow!("parse_sql_with_error error")),
    }
}

async fn parse_insert_sql(
    mut rows_event: RowsEventRows<'_>,
    db_name: String,
    table_name: String,
    column_list: Vec<String>,
) -> Result<String, anyhow::Error> {
    // let row_event = write_rows_event.rows.first().ok_or(anyhow!("no rows"))?;
    let mut column_names = vec![];
    let mut column_values = vec![];
    let mut res = "".to_string();
    while let Some(item) = rows_event.next() {
        match item {
            Ok((row1, row2)) => match (row1, row2) {
                (None, Some(mut r2)) => {
                    let columns = r2.columns();
                    for (index, item) in columns.iter().enumerate() {
                        column_names.push(column_list[index].clone());

                        let value = r2.take(index);
                        if let Some(v) = value {
                            match v {
                                BinlogValue::Value(v) => match v {
                                    Value::Int(i) => {
                                        column_values.push(format!("{}", i));
                                    }
                                    Value::Bytes(s) => {
                                        column_values
                                            .push(format!("\"{}\"", String::from_utf8_lossy(&s)));
                                    }
                                    Value::Double(v) => {
                                        column_values.push("NULL".to_string());
                                    }
                                    Value::Float(v) => {}
                                    Value::UInt(v) => {
                                        column_values.push(format!("{}", v));
                                    }
                                    Value::NULL => {
                                        column_values.push("NULL".to_string());
                                    }
                                    _ => {}
                                },
                                BinlogValue::Jsonb(aaa) => {
                                    info!("jsonb: {:?}", aaa);
                                }
                                BinlogValue::JsonDiff(ss) => {
                                    info!("jsondiff: {:?}", ss);
                                }
                            }
                        }
                    }
                }
                (Some(r1), Some(r2)) => {
                    info!("r1:{:?},r2:{:?}", r1, r2);
                }
                (Some(r1), None) => {
                    info!("r1:{:?},", r1);
                }

                _ => {}
            },
            Err(e) => {
                // Handle the error case
                info!("An error occurred: {:?}", e);
            }
        }
    }
    res = format!(
        "INSERT INTO `{}`.`{}`({}) VALUES ({});",
        db_name,
        table_name,
        column_names.join(" , "),
        column_values.join(" , ")
    );
    info!("{}", res);
    Ok(res)
}
async fn parse_update_sql(
    mut rows_event: RowsEventRows<'_>,
    db_name: String,
    table_name: String,
    column_list: Vec<String>,
) -> Result<String, anyhow::Error> {
    let mut before_values = vec![];
    let mut after_values = vec![];
    let mut column_names = vec![];

    let mut res = "".to_string();
    while let Some(item) = rows_event.next() {
        match item {
            Ok((row1, row2)) => match (row1, row2) {
                (Some(mut r1), Some(mut r2)) => {
                    let columns = r2.columns();
                    for (index, item) in columns.iter().enumerate() {
                        column_names.push(column_list[index].clone());
                        let before_value = r1.take(index);

                        let after_value = r2.take(index);
                        if let (Some(before_value), Some(after_value)) = (before_value, after_value)
                        {
                            match (before_value, after_value) {
                                (
                                    BinlogValue::Value(Value::Int(before_i)),
                                    BinlogValue::Value(Value::Int(after_i)),
                                ) => {
                                    let before_value =
                                        format!(r#"{}={}"#, column_list[index], before_i);
                                    let after_value =
                                        format!(r#"{}={}"#, column_list[index], after_i);
                                    before_values.push(before_value);
                                    after_values.push(after_value);
                                }
                                (
                                    BinlogValue::Value(Value::Bytes(before_i)),
                                    BinlogValue::Value(Value::Bytes(after_i)),
                                ) => {
                                    let before_value = format!(
                                        r#"{}={}"#,
                                        column_list[index],
                                        String::from_utf8_lossy(&before_i)
                                    );
                                    let after_value = format!(
                                        r#"{}="{}""#,
                                        column_list[index],
                                        String::from_utf8_lossy(&after_i)
                                    );
                                    before_values.push(before_value);
                                    after_values.push(after_value);
                                }
                                (
                                    BinlogValue::Value(Value::Double(before_i)),
                                    BinlogValue::Value(Value::Double(after_i)),
                                ) => {}
                                (
                                    BinlogValue::Value(Value::UInt(before_i)),
                                    BinlogValue::Value(Value::UInt(after_i)),
                                ) => {}
                                _ => {}
                            }
                        }
                    }
                }

                (Some(r1), None) => {
                    println!("r1:{:?},", r1);
                }

                _ => {}
            },
            Err(e) => {
                // Handle the error case
                println!("An error occurred: {:?}", e);
            }
        }
    }
    res = format!(
        "UPDATE `{}`.`{}` SET {} WHERE {} LIMIT 1;",
        db_name,
        table_name,
        before_values.join(" , "),
        after_values.join(" AND  ")
    );
    info!("{}", res);
    Ok(res)
}
async fn parse_delete_sql(
    mut rows_event: RowsEventRows<'_>,
    db_name: String,
    table_name: String,
    column_list: Vec<String>,
) -> Result<String, anyhow::Error> {
    let mut column_names = vec![];
    let mut column_values = vec![];
    let mut res = "".to_string();
    while let Some(item) = rows_event.next() {
        match item {
            Ok((row1, row2)) => match (row1, row2) {
                (Some(mut r1), None) => {
                    let columns = r1.columns();
                    for (index, item) in columns.iter().enumerate() {
                        column_names.push(column_list[index].clone());

                        let value = r1.take(index);
                        if let Some(v) = value {
                            match v {
                                BinlogValue::Value(v) => match v {
                                    Value::Int(i) => {
                                        let current = format!(r#"{}={}"#, column_list[index], i);
                                        column_values.push(current);
                                    }
                                    Value::Bytes(s) => {
                                        let value = String::from_utf8_lossy(&s);
                                        let current =
                                            format!(r#"{}="{}""#, column_list[index], value);
                                        column_values.push(current);
                                    }
                                    Value::Double(v) => {
                                        // column_values.push("NULL".to_string());
                                    }
                                    Value::Float(v) => {}
                                    Value::UInt(v) => {
                                        // column_values.push(format!("{}", v));
                                    }
                                    Value::NULL => {}
                                    _ => {}
                                },
                                BinlogValue::Jsonb(aaa) => {
                                    info!("jsonb: {:?}", aaa);
                                }
                                BinlogValue::JsonDiff(ss) => {
                                    info!("jsondiff: {:?}", ss);
                                }
                            }
                        }
                    }
                }
                (Some(r1), Some(r2)) => {
                    info!("r1:{:?},r2:{:?}", r1, r2);
                }
                (Some(r1), None) => {
                    info!("r1:{:?},", r1);
                }

                _ => {}
            },
            Err(e) => {
                // Handle the error case
                println!("An error occurred: {:?}", e);
            }
        }
    }
    res = format!(
        "DELETE FROM `{}`.`{}` WHERE {} LIMIT 1;",
        db_name,
        table_name,
        column_values.join(" AND "),
    );
    info!("{}", res);
    Ok(res)
}
