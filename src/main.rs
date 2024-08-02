use anyhow::anyhow;

use futures::StreamExt;
use moka::future::Cache;
use mysql_async::binlog::events::{RowsEventData, TableMapEvent, WriteRowsEvent};
use mysql_async::binlog::value::BinlogValue;
use mysql_common::proto::MySerialize;
use service::database_service::get_database_list;
use service::datasource_service::{create_datasource, get_datasource_list};
mod common;
mod dao;
use service::table_service::get_table_list;
use tokio::time::{sleep, Sleep};
use tracing_subscriber::fmt::Layer as FmtLayer;

use crate::common::init::init_with_error;
mod service;
use tracing_subscriber::Layer;
mod util;
mod vojo;
use axum::routing::get;
use axum::routing::post;
use axum::Router;

use rand::{seq::IteratorRandom, thread_rng}; // 0.6.1
use sqlx::mysql::MySqlConnection;
use sqlx::mysql::MySqlPoolOptions;
use std::io::{Read, Write};

use sqlx::{any, Connection, Row};
use std::{collections::HashMap, hash::Hash, vec};
use tracing_appender::non_blocking::NonBlockingBuilder;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_appender::rolling;
use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
#[macro_use]
extern crate tracing;
#[macro_use]
extern crate anyhow;
async fn create_data() -> Result<(), anyhow::Error> {
    let pool = MySqlPoolOptions::new()
        .max_connections(5)
        .connect("mysql://root:root@localhost:9306/mydb")
        .await?;

    for i in 0..1000 {
        let i_str = i.to_string();
        let i_next_str = (i + 1).to_string();
        sqlx::query("insert into user(username,first_name,content)values(?,?,?)")
            .bind(i_str.clone())
            .bind(i_str.clone())
            .bind(i_str.clone())
            .execute(&pool)
            .await?;
        sqlx::query("update user set first_name = ? where username=?")
            .bind(i_next_str)
            .bind(i_str.clone())
            .execute(&pool)
            .await?;
        sqlx::query("delete from user").execute(&pool).await?;
    }

    Ok(())
}
fn setup_logger() -> Result<WorkerGuard, anyhow::Error> {
    let app_file = rolling::daily("./logs", "access.log");
    let (non_blocking_appender, guard) = NonBlockingBuilder::default()
        .buffered_lines_limit(10)
        .finish(app_file);
    let file_layer = tracing_subscriber::fmt::Layer::new()
        .with_target(true)
        .with_ansi(false)
        .with_writer(non_blocking_appender)
        .with_filter(tracing_subscriber::filter::LevelFilter::INFO);
    let console_layer = FmtLayer::new()
        .with_target(true)
        .with_ansi(true)
        .with_filter(tracing_subscriber::filter::LevelFilter::TRACE);
    tracing_subscriber::registry()
        .with(file_layer)
        .with(console_layer)
        .with(tracing_subscriber::filter::LevelFilter::TRACE)
        .init();
    Ok(guard)
}
#[tokio::main]
async fn main() {
    if let Err(e) = test_binlog_with_realtime().await {
        println!("{:?}", e);
    }
}
async fn main_with_error() -> Result<(), anyhow::Error> {
    let _work_guard = setup_logger()?;
    let db_pool = common::sql_connections::create_pool().await?;
    init_with_error(db_pool.clone()).await?;
    let app = Router::new()
        .route(
            "/datasource",
            post(create_datasource).get(get_datasource_list),
        )
        .route("/datasource/:id/database/:name/tables", get(get_table_list))
        .route("/datasource/:id", get(get_database_list))
        .with_state(db_pool);
    let final_route = Router::new().nest("/api", app);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:9394").await.unwrap();
    axum::serve(listener, final_route).await.unwrap();
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

use mysql_async::{Conn, Sid};
use mysql_async::{Opts, Value};
use mysql_common::binlog::events::EventData;
use std::time::Duration;
async fn test_binlog_with_realtime() -> Result<(), anyhow::Error> {
    let cache: Cache<String, Vec<String>> = Cache::new(10_000);
    println!("a");
    let mysql = Conn::new(Opts::from_url("mysql://root:root@127.0.0.1:9306")?).await?;
    let input = "e914a9a2-4ff6-11ef-9eca-0242ac130002:3014";
    let sid = input.parse::<Sid>()?;

    // let e = input.parse::<Sid>().unwrap_err();
    let mut stream = mysql
        .get_binlog_stream(
            mysql_async::BinlogStreamRequest::new(11114)
                .with_gtid()
                .with_gtid_set(vec![]),
        )
        .await?;
    println!("a1");

    let mut bin_log_name = "".to_string();
    let mut bin_log_position = 0;

    while let Some(Ok(data)) = stream.next().await {
        // println!("header>>>:{:?}", data);
        bin_log_position = data.header().log_pos();

        let event_data = data.read_data()?.ok_or(anyhow!("sssss"))?;

        match event_data {
            EventData::QueryEvent(query) => {
                println!("{}", String::from_utf8_lossy(query.query_raw()));
            }
            EventData::TableMapEvent(table_map_event) => {
                let db_name = table_map_event.database_name();
                let table_name = table_map_event.table_name();
                if db_name == "mysql"
                    || db_name == "performance_schema"
                    || db_name == "information_schema"
                    || db_name == "sys"
                {
                    continue;
                }
                let key = format!("{}{}", db_name, table_name);
                let s = cache.get(&key).await;

                let column_list = if s.is_none() {
                    let v =
                        parse_colomns(db_name.to_string().clone(), table_name.to_string().clone())
                            .await?;
                    cache.insert(db_name.to_string().clone(), v.clone()).await;
                    v
                } else {
                    s.unwrap()
                };
                if let Some(Ok(new_data)) = stream.next().await {
                    let new_event_data = new_data.read_data()?.ok_or(anyhow!("sssss"))?;

                    parse_sql_with_error(new_event_data, table_map_event, column_list).await?;
                }
            }
            EventData::GtidEvent(gtid_event) => {
                // println!("{:?}", gtid_event);
                let gtid = uuid::Uuid::from_bytes(gtid_event.sid());
                println!(
                    "{}-{}-gtid:=============================={}:{}",
                    bin_log_name,
                    bin_log_position,
                    gtid.to_string(),
                    gtid_event.gno()
                );
            }
            EventData::XidEvent(e) => {
                println!("{}-{}-COMMIT;", bin_log_name, bin_log_position);
            }
            EventData::RotateEvent(rotate_event) => {
                bin_log_name = rotate_event.name().to_string();
                println!("rotate_event>>>{:?}", rotate_event);
            }
            _ => {
                // println!("other>>>{:?}", event_data);
            }
        }
        // sleep(Duration::from_millis(1000)).await;
    }
    Ok(())
}
async fn parse_sql_with_error(
    data: EventData<'_>,
    table_map_event: TableMapEvent<'_>,
    column_list: Vec<String>,
    // db_name: String,
    // table_name: String,
) -> Result<(), anyhow::Error> {
    match data {
        EventData::RowsEvent(rows_event_data) => match rows_event_data {
            RowsEventData::WriteRowsEvent(write_rows_event) => {
                parse_insert_sql(write_rows_event, table_map_event, column_list).await?;
            }
            RowsEventData::UpdateRowsEvent(update_rows_event) => {
                println!("update_rows_event: {:?}", update_rows_event);
            }
            RowsEventData::DeleteRowsEvent(delete_rows_event) => {
                println!("delete_rows_event: {:?}", delete_rows_event);
            }
            _ => {}
        },

        _ => {}
    }
    Ok(())
}
async fn parse_insert_sql(
    write_rows_event: WriteRowsEvent<'_>,
    table_map_event: TableMapEvent<'_>,
    column_list: Vec<String>,
) -> Result<(), anyhow::Error> {
    let mut rows_event = write_rows_event.rows(&table_map_event);
    // let row_event = write_rows_event.rows.first().ok_or(anyhow!("no rows"))?;
    let mut column_names = vec![];
    let mut column_values = vec![];
    let db_name = table_map_event.database_name();
    let table_name = table_map_event.table_name();
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
                                    println!("jsonb: {:?}", aaa);
                                }
                                BinlogValue::JsonDiff(ss) => {
                                    println!("jsondiff: {:?}", ss);
                                }
                            }
                        }
                    }
                }
                _ => {}
            },
            Err(e) => {
                // Handle the error case
                println!("An error occurred: {:?}", e);
            }
        }
    }
    let res = format!(
        "INSERT INTO `{}`.`{}`({}) VALUES ({});",
        db_name,
        table_name,
        column_names.join(" , "),
        column_values.join(" , ")
    );
    println!("{}", res);
    Ok(())
}
