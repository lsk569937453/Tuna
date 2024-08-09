use crate::dao::task_dao::TaskDao;
use anyhow::anyhow;
use futures::StreamExt;
use moka::future::Cache;
use mysql_async::binlog::events::EventData;
use mysql_async::binlog::events::TableMapEvent;
use mysql_async::binlog::events::UpdateRowsEvent;
use mysql_async::binlog::value::BinlogValue;
use mysql_async::BinlogStream;
use mysql_async::{Conn, Sid};
use mysql_async::{Opts, Value};
use sqlx::Connection;

use mysql_async::binlog::events::{RowsEvent, RowsEventData, RowsEventRows, WriteRowsEvent};

use sqlx::mysql::MySqlConnection;
use sqlx::Row;

pub struct BinlogPoller {
    pub gtid_set: Option<String>,
    binlog_stream: BinlogStream,
    current_db_name: String,
    current_binlog_name: String,
    current_binlog_position: u32,
    current_table_map_event: Option<TableMapEvent<'static>>,
    column_list: Option<Vec<String>>,
    cache: Cache<String, Vec<String>>,
}
//impl debug for me
impl std::fmt::Debug for BinlogPoller {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BinlogPoller")
            .field("gtid_set", &self.gtid_set)
            .field("current_db_name", &self.current_db_name)
            .field("current_binlog_name", &self.current_binlog_name)
            .field("current_binlog_position", &self.current_binlog_position)
            .finish()
    }
}
impl BinlogPoller {
    pub async fn start() -> Result<Self, anyhow::Error> {
        let datasource = "self.task_dao.from_datasource_id";
        let mysql = Conn::new(Opts::from_url("mysql://root:root@127.0.0.1:9306")?).await?;
        let input = "ce9d6204-5530-11ef-a4c8-0242c0a83002:1-3000";
        let sid = input.parse::<Sid>()?;

        // let e = input.parse::<Sid>().unwrap_err();
        let mut stream = mysql
            .get_binlog_stream(
                mysql_async::BinlogStreamRequest::new(11)
                    .with_gtid()
                    .with_gtid_set(vec![sid]),
            )
            .await?;

        let sel = Self {
            gtid_set: None,
            binlog_stream: stream,
            current_db_name: String::new(),
            current_binlog_name: String::new(),
            current_binlog_position: 0,
            current_table_map_event: None,
            column_list: None,
            cache: Cache::new(1000),
        };
        Ok(sel)
    }
    #[instrument]
    pub async fn poll(&mut self) -> Result<(), anyhow::Error> {
        if let Some(Ok(event)) = self.binlog_stream.next().await {
            self.current_binlog_position = event.header().log_pos();
            let event_cloned = event.clone();
            let option_event_data = event_cloned.read_data()?;
            let event_data = option_event_data.ok_or(anyhow!("Read data error"))?;
            let dst = event_data.clone();

            match dst {
                EventData::TableMapEvent(table_map_event) => {
                    let db_name = table_map_event.database_name();
                    let table_name = table_map_event.table_name();
                    let key = format!("{}{}", db_name, table_name);
                    if db_name != "mydb" {
                        // println!(">>>>>>>>>>>>>>>>>>>{}-{}", db_name, table_name);
                        return Ok(());
                    }
                    let s = self.cache.get(&key).await;

                    //可能查询很多次
                    let current_column_list = if s.is_none() {
                        let v = parse_colomns(
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
                }
                EventData::RowsEvent(rows_event_data) => {
                    if !should_save(self.current_table_map_event.clone()) {
                        return Ok(());
                    }
                    let table_map_eventt = self.current_table_map_event.clone();
                    let data = table_map_eventt.ok_or(anyhow!(""))?;
                    let column_list = self.column_list.clone().ok_or(anyhow!(""))?;
                    parse_sql_with_error(rows_event_data, data.clone(), column_list).await?;
                }
                EventData::QueryEvent(query_event) => {
                    info!(
                        "QueryEvent:{}",
                        String::from_utf8_lossy(query_event.query_raw()),
                    );
                }
                EventData::RowsQueryEvent(rows_query_event) => {
                    info!("{:?}", rows_query_event);
                }
                EventData::TransactionPayloadEvent(transaction_payload_event) => {
                    info!("{:?}", transaction_payload_event);
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
                }
                EventData::XidEvent(e) => {
                    info!(
                        "{}-{}-COMMIT;",
                        self.current_binlog_name, self.current_binlog_position,
                    );
                }
                EventData::RotateEvent(rotate_event) => {
                    self.current_binlog_name = rotate_event.name().to_string();
                    info!("rotate_event>>>{:?}", rotate_event);
                }
                _ => {
                    info!("other>>>>>{:?}", event_data);
                }
            }
            // sleep(Duration::from_millis(100)).await;
        }
        Ok(())
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
    // db_name: String,
    // table_name: String,
) -> Result<(), anyhow::Error> {
    let db_name = table_map_event.database_name().to_string();
    let table_name = table_map_event.table_name().to_string();
    match rows_event_data {
        RowsEventData::WriteRowsEvent(write_rows_event) => {
            let rows_event = write_rows_event.rows(&table_map_event);
            let (column_names, column_values) =
                parse_insert_sql(rows_event, db_name, table_name, column_list).await?;
        }
        RowsEventData::UpdateRowsEvent(update_rows_event) => {
            let rows_event = update_rows_event.rows(&table_map_event);
            parse_update_sql(rows_event, db_name, table_name, column_list).await?;
        }
        RowsEventData::DeleteRowsEvent(delete_rows_event) => {
            let rows_event = delete_rows_event.rows(&table_map_event);
            parse_delete_sql(rows_event, db_name, table_name, column_list).await?;
        }
        _ => {}
    }

    Ok(())
}
async fn parse_colomns(database: String, table_name: String) -> Result<Vec<String>, anyhow::Error> {
    let mut conn = MySqlConnection::connect("mysql://root:root@localhost:9306").await?;
    let  rows=sqlx::query("SELECT COLUMN_NAME FROM INFORMATION_SCHEMA.COLUMNS WHERE TABLE_SCHEMA = ? AND TABLE_NAME = ? ORDER BY ORDINAL_POSITION")
    .bind(database)
    .bind(table_name)
    .fetch_all(&mut conn)
    .await?;
    let mut res = vec![];
    for it in rows.iter() {
        let item: String = it.get(0);
        res.push(item);
    }
    // println!("res: {:?}", res);
    Ok(res)
}
async fn parse_insert_sql(
    mut rows_event: RowsEventRows<'_>,
    db_name: String,
    table_name: String,
    column_list: Vec<String>,
) -> Result<(String, String), anyhow::Error> {
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
    Ok((column_names.join(","), column_values.join(" , ")))
}
async fn parse_update_sql(
    mut rows_event: RowsEventRows<'_>,
    db_name: String,
    table_name: String,
    column_list: Vec<String>,
) -> Result<(), anyhow::Error> {
    // let row_event = write_rows_event.rows.first().ok_or(anyhow!("no rows"))?;
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
    Ok(())
}
async fn parse_delete_sql(
    mut rows_event: RowsEventRows<'_>,
    db_name: String,
    table_name: String,
    column_list: Vec<String>,
) -> Result<(String, String), anyhow::Error> {
    // let row_event = write_rows_event.rows.first().ok_or(anyhow!("no rows"))?;
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
    Ok((column_names.join(","), column_values.join(" , ")))
}
