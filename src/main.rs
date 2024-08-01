use anyhow::anyhow;

use futures::StreamExt;
use moka::future::Cache;
use mysql_common::proto::MySerialize;
use service::database_service::get_database_list;
use service::datasource_service::{create_datasource, get_datasource_list};
mod common;
mod dao;
use service::table_service::get_table_list;
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
use std::io::Write;

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
// async fn test_binlog_with_file() -> Result<(), anyhow::Error> {
//     let file_path = "d:/mysql-bin.000003";
//     let mut file = std::fs::File::open(file_path).unwrap();

//     let mut parser = BinlogParser {
//         checksum_length: 4,
//         table_map_event_by_table_id: HashMap::new(),
//     };

//     assert!(parser.check_magic(&mut file).is_ok());
//     loop {
//         if let Ok((header, data)) = parser.next(&mut file) {
//             println!("header: {:?}", header);
//             println!("data: {:?}", data);
//             parse_json_columns(data);

//             println!("");
//         } else {
//             println!("parse error")
//         }
//     }
//     let sql="SELECT COLUMN_NAME FROM INFORMATION_SCHEMA.COLUMNS WHERE TABLE_SCHEMA = ? AND TABLE_NAME = ? ORDER BY ORDINAL_POSITION";
//     Ok(())
// }
// pub fn as_the_bytes(
//     server_id: i32,
//     gtid_set: String,
//     file_name: String,
// ) -> Result<Vec<u8>, anyhow::Error> {
//     let mut buf = Vec::new();

//     // Command byte for COM_BINLOG_DUMP_GTID
//     buf.write_u8(CommandType::BinlogDumpGtid as u8)?;

//     // Flags
//     let binlog_flags = 0;
//     buf.write_u32::<LittleEndian>(binlog_flags)?;

//     // Server-ID
//     buf.write_u32::<LittleEndian>(server_id as u32)?;

//     buf.write_all(file_name.as_bytes())?;

//     // GTID Set
//     let gtid_set_bytes = gtid_set.as_bytes();
//     let gtid_set_len = gtid_set_bytes.len() as u32;

//     // Data Size
//     buf.write_u32::<LittleEndian>(gtid_set_len)?;

//     // GTID Set data
//     buf.write_all(gtid_set_bytes)?;

//     Ok(buf)
// }
// async fn connect_myself(
//     url: String,
//     server_id: i32,
//     gtid_set: String,
//     file_name: String,
// ) -> Result<BinlogStream, anyhow::Error> {
//     let mut authenticator =
//         Authenticator::new(&url).map_err(|e| anyhow!("Authenticator error{}", e))?;
//     let mut channel = authenticator
//         .connect()
//         .await
//         .map_err(|e| anyhow!("channel connect error{}", e))?;
//     // fetch binlog checksum
//     let binlog_checksum = CommandUtil::fetch_binlog_checksum(&mut channel)
//         .await
//         .map_err(|e| anyhow!("fetch_binlog_checksum{}", e))?;
//     println!("aa");
//     // setup connection
//     CommandUtil::setup_binlog_connection(&mut channel)
//         .await
//         .map_err(|e| anyhow!("setup_binlog_connection {}", e))?;
//     println!("aa2");

//     let buf = as_the_bytes(server_id, gtid_set, file_name)?;
//     println!("aa3");

//     channel
//         .write(&buf, 0)
//         .await
//         .map_err(|e| anyhow!("channel write error{}", e))?;
//     println!("aa4");

//     let parser = BinlogParser {
//         checksum_length: binlog_checksum.get_length(),
//         table_map_event_by_table_id: HashMap::new(),
//     };

//     Ok(BinlogStream { channel, parser })
// }
use mysql_async::Conn;
use mysql_async::Opts;
use mysql_common::binlog::events::EventData;
async fn test_binlog_with_realtime() -> Result<(), anyhow::Error> {
    let cache: Cache<String, Vec<String>> = Cache::new(10_000);
    let mut mysql = Conn::new(Opts::from_url("mysql://root:root@127.0.0.1:9306")?).await?;
    let input = "3E11FA47-71CA-11E1-9E33-C80AA9429562:1-5:10-15:20-";
    // let e = input.parse::<Sid>().unwrap_err();
    let mut stream = mysql
        .get_binlog_stream(
            mysql_async::BinlogStreamRequest::new(11114)
                .with_gtid()
                .with_gtid_set(vec![]),
        )
        .await?;
    // let mut client = BinlogClient {
    //     url: String::from("mysql://root:root@127.0.0.1:9306"),
    //     binlog_filename: "mysql-binlog.000001".to_string(),
    //     binlog_position: 0,
    //     server_id: 1,
    // };
    // let mut stream = client.connect().await?;

    let mut count = 0;
    while let Some(Ok(data)) = stream.next().await {
        let event_data = data.read_data()?.unwrap();

        println!("{:?}", event_data);
        match event_data {
            EventData::QueryEvent(query) => {
                println!("query: {:?}", 1);
            }
            EventData::TableMapEvent(table_map) => {
                println!("table_map: {:?}", table_map);
            }
            EventData::GtidEvent(gtid_event) => {
                let t = gtid_event.sid();

                println!(
                    "gtid:============================== {:?}",
                    String::from_utf8_lossy(&t)
                );
            }
            _ => {}
        }
        if count % 10 == 0 {
            println!("count is :{}", count);
        }
    }
    Ok(())
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
