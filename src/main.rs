use anyhow::anyhow;

use clap::builder::Str;
use futures::TryStreamExt;
use moka::future::Cache;
use mysql_binlog_connector_rust::binlog_parser::BinlogParser;
use mysql_binlog_connector_rust::event::delete_rows_event::DeleteRowsEvent;
use mysql_binlog_connector_rust::event::update_rows_event::UpdateRowsEvent;
use mysql_binlog_connector_rust::event::write_rows_event::WriteRowsEvent;
use mysql_binlog_connector_rust::{
    binlog_client::BinlogClient,
    column::{column_value::ColumnValue, json::json_binary::JsonBinary},
    event::{event_data::EventData, row_event::RowEvent},
};
use service::database_service::get_database_list;
use service::datasource_service::create_datasource;
mod common;
mod dao;
use crate::common::init::init_with_error;
mod service;
mod vojo;
use axum::routing::get;
use axum::routing::post;
use axum::Router;
use rand::{seq::IteratorRandom, thread_rng}; // 0.6.1
use sqlx::mysql::MySqlConnection;
use sqlx::mysql::MySqlPoolOptions;
use sqlx::{any, Connection, Row};
use std::{collections::HashMap, hash::Hash, vec};
use tracing_appender::non_blocking::NonBlockingBuilder;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_appender::rolling;
use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::Layer;
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

    tracing_subscriber::registry()
        .with(file_layer)
        .with(tracing_subscriber::filter::LevelFilter::TRACE)
        .init();
    Ok(guard)
}
#[tokio::main]
async fn main() {
    if let Err(e) = main_with_error().await {
        println!("{:?}", e);
    }
}
async fn main_with_error() -> Result<(), anyhow::Error> {
    let _work_guard = setup_logger()?;
    let db_pool = common::sql_connections::create_pool().await?;
    init_with_error(db_pool.clone()).await?;
    let app = Router::new()
        .route("/datasource", post(create_datasource))
        .route("/database/:id", get(get_database_list))
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
async fn test_binlog_with_file() -> Result<(), anyhow::Error> {
    let file_path = "d:/mysql-bin.000003";
    let mut file = std::fs::File::open(file_path).unwrap();

    let mut parser = BinlogParser {
        checksum_length: 4,
        table_map_event_by_table_id: HashMap::new(),
    };

    assert!(parser.check_magic(&mut file).is_ok());
    loop {
        if let Ok((header, data)) = parser.next(&mut file) {
            println!("header: {:?}", header);
            println!("data: {:?}", data);
            parse_json_columns(data);

            println!("");
        } else {
            println!("parse error")
        }
    }
    let sql="SELECT COLUMN_NAME FROM INFORMATION_SCHEMA.COLUMNS WHERE TABLE_SCHEMA = ? AND TABLE_NAME = ? ORDER BY ORDINAL_POSITION";
    Ok(())
}
async fn test_binlog_with_realtime() -> Result<(), anyhow::Error> {
    let cache: Cache<String, Vec<String>> = Cache::new(10_000);

    let mut client = BinlogClient {
        url: String::from("mysql://root:root@127.0.0.1:9306"),
        binlog_filename: "mysql-binlog.000001".to_string(),
        binlog_position: 0,
        server_id: 1,
    };

    let mut stream = client.connect().await?;
    let mut count = 0;
    loop {
        let (header, data) = stream.read().await?;
        match data {
            EventData::Query(t) => println!("{};", t.query),
            EventData::TableMap(t) => {
                // println!("table event:{:?}", t);

                let db_name = t.database_name;
                let table_name = t.table_name;
                if db_name == "mysql" {
                    continue;
                }
                let key = format!("{}{}", db_name, table_name);
                let s = cache.get(&key).await;

                let column_list = if s.is_none() {
                    let v = parse_colomns(db_name.clone(), table_name.clone()).await?;
                    cache.insert(db_name.clone(), v.clone()).await;
                    v
                } else {
                    s.unwrap()
                };
                let (_, new_data) = stream.read().await?;
                parse_sql_with_error(new_data, column_list, db_name, table_name).await?;
                count += 1;
            }
            EventData::Gtid(t) => println!("gtid:{:?}", t),
            EventData::Xid(_) => println!("END;"),
            _ => {}
        }
        if count % 10 == 0 {
            println!("count is :{}", count);
        }
    }
    Ok(())
}
async fn parse_sql_with_error(
    data: EventData,
    column_list: Vec<String>,
    db_name: String,
    table_name: String,
) -> Result<(), anyhow::Error> {
    match data {
        EventData::WriteRows(write_rows_event) => {
            parse_insert_sql(write_rows_event, column_list, db_name, table_name).await?;
        }
        EventData::UpdateRows(update_rows_event) => {
            parse_update_sql(update_rows_event, column_list, db_name, table_name).await?;
        }
        EventData::DeleteRows(delete_rows_event) => {
            parse_delete_sql(delete_rows_event, column_list, db_name, table_name).await?
        }
        _ => {}
    }
    Ok(())
}
async fn parse_insert_sql(
    write_rows_event: WriteRowsEvent,
    column_list: Vec<String>,
    db_name: String,
    table_name: String,
) -> Result<(), anyhow::Error> {
    let row_event = write_rows_event.rows.first().ok_or(anyhow!("no rows"))?;
    let mut column_name = vec![];
    let mut column_value = vec![];
    for (index, item) in row_event.column_values.iter().enumerate() {
        match item {
            ColumnValue::Long(v) => {
                column_name.push(column_list[index].clone());
                column_value.push(format!("{}", v));
            }
            ColumnValue::LongLong(v) => {
                column_name.push(column_list[index].clone());
                column_value.push(format!("{}", v));
            }
            ColumnValue::String(v) => {
                column_name.push(column_list[index].clone());
                column_value.push(format!(r#""{}""#, String::from_utf8_lossy(v)));
            }
            ColumnValue::None => {
                continue;
            }
            ColumnValue::Blob(s) => {
                continue;
            }
            _ => {
                println!("The app do not parse the type: {:?}", item);
            }
        }
    }
    let res = format!(
        "INSERT INTO `{}`.`{}`({}) VALUES ({});",
        db_name,
        table_name,
        column_name.join(" , "),
        column_value.join(" , ")
    );
    println!("{}", res);
    Ok(())
}
async fn parse_update_sql(
    update_rows_event: UpdateRowsEvent,
    column_list: Vec<String>,
    db_name: String,
    table_name: String,
) -> Result<(), anyhow::Error> {
    let (before_update_rows, after_update_rows) =
        update_rows_event.rows.first().ok_or(anyhow!("no rows"))?;

    let before_vec = parse_update_event(column_list.clone(), before_update_rows.clone()).await?;
    let after_vec = parse_update_event(column_list, after_update_rows.clone()).await?;
    let before_str = before_vec.join(" , ");
    let after_str = after_vec.join(" AND ");
    let res = format!(
        "UPDATE `{}`.`{}` SET {} WHERE {} LIMIT 1;",
        db_name, table_name, before_str, after_str
    );
    println!("{}", res);
    Ok(())
}
async fn parse_delete_sql(
    delete_rows_event: DeleteRowsEvent,
    column_list: Vec<String>,
    db_name: String,
    table_name: String,
) -> Result<(), anyhow::Error> {
    let row_event = delete_rows_event.rows.first().ok_or(anyhow!("no rows"))?;
    let mut res = vec![];
    for (index, item) in row_event.column_values.iter().enumerate() {
        match item {
            ColumnValue::Long(v) => {
                let current = format!(r#"{}={}"#, column_list[index], v);
                res.push(current);
            }
            ColumnValue::String(v) => {
                let value = String::from_utf8_lossy(v);
                let current = format!(r#"{}="{}""#, column_list[index], value);
                res.push(current);
            }
            ColumnValue::None => {
                continue;
            }
            _ => {
                println!("The app do not parse the type: {:?}", item);
            }
        }
    }
    let result = format!(
        "DELETE FROM `{}`.`{}` WHERE {} LIMIT 1;",
        db_name,
        table_name,
        res.join(" AND "),
    );
    println!("{}", result);

    Ok(())
}
async fn parse_update_event(
    column_list: Vec<String>,
    row_event: RowEvent,
) -> Result<Vec<String>, anyhow::Error> {
    let mut res = vec![];
    for (index, item) in row_event.column_values.iter().enumerate() {
        match item {
            ColumnValue::Long(v) => {
                let current = format!(r#"{}={}"#, column_list[index], v);
                res.push(current);
            }
            ColumnValue::String(v) => {
                let value = String::from_utf8_lossy(v);
                let current = format!(r#"{}="{}""#, column_list[index], value);
                res.push(current);
            }
            ColumnValue::None => {
                continue;
            }
            _ => {
                println!("The app do not parse the type: {:?}", item);
            }
        }
    }
    Ok(res)
}
fn parse_json_columns(data: EventData) {
    let parse_row = |row: RowEvent| {
        for column_value in row.column_values {
            if let ColumnValue::Json(bytes) = column_value {
                println!(
                    "json column: {}",
                    JsonBinary::parse_as_string(&bytes).unwrap()
                )
            } else if let ColumnValue::Long(lon) = column_value {
                println!("lon is:{}", lon);
            } else if let ColumnValue::String(t) = column_value {
                println!("t is :{}", String::from_utf8_lossy(&t));
            }
        }
    };

    match data {
        EventData::WriteRows(event) => {
            for row in event.rows {
                parse_row(row)
            }
        }
        EventData::DeleteRows(event) => {
            for row in event.rows {
                parse_row(row)
            }
        }
        EventData::UpdateRows(event) => {
            for (before, after) in event.rows {
                parse_row(before);
                parse_row(after);
            }
        }
        _ => {}
    }
}
fn test() -> Result<(), anyhow::Error> {
    let mut v = vec![
        "red-1", "red-2", "red-3", "red-4", "red-5", "blue-1", "blue-2", "blue-3", "blue-4",
        "blue-5", "yellow-1", "yellow-2", "yellow-3", "yellow-4", "yellow-5",
    ];
    let mut result_0 = 0;
    let mut result_1 = 0;
    for i in 0..1000 {
        let res = test_1(v.clone())?;
        if res == 0 {
            result_0 += 1;
        } else {
            result_1 += 1;
        }
    }
    println!("result_0: {}, result_1: {}", result_0, result_1);
    Ok(())
}
fn test_1(v: Vec<&str>) -> Result<i32, anyhow::Error> {
    let mut rng = thread_rng();
    let sample = v.iter().choose_multiple(&mut rng, 12);
    let mut hashmap = HashMap::new();
    for item in sample {
        let key = item.split("-").next().ok_or(anyhow!("error"))?;
        if let Some(count) = hashmap.get(&key) {
            hashmap.insert(key, count + 1);
        } else {
            hashmap.insert(key, 1);
        };
    }
    let mut hash_vec: Vec<_> = hashmap.iter().collect();
    hash_vec.sort_by(|a, b| a.1.cmp(b.1));
    if (*hash_vec[0].1) == 3 && (*hash_vec[1].1) == 4 && (*hash_vec[2].1) == 5 {
        return Ok(1);
    }
    //print the hash_vec
    // for item in hash_vec {
    //     print!("{}: {}", item.0, item.1);
    // }
    // println!();
    Ok(0)
}
