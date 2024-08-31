use anyhow::anyhow;

use binlog::binlog_realtime::test_binlog_with_realtime;
use common::init_redis;
use futures::StreamExt;
use moka::future::Cache;
use mysql_async::binlog::events::{RowsEventData, RowsEventRows, TableMapEvent};
use mysql_async::binlog::value::BinlogValue;
use schedule::sync_redis::main_sync_redis_loop_with_error;
use service::audit_task_result_service::get_audit_tasks_result;
use service::audit_task_service::{create_audit_task, execute_audit_task, get_audit_tasks};
use service::database_service::get_database_list;
use service::datasource_service::{
    create_datasource, delete_datasource_by_id, get_datasource_list,
};
mod common;
mod dao;
use service::table_service::get_table_list;
use service::task_servivce::{create_task, get_task_list};
use tracing_subscriber::fmt::Layer as FmtLayer;
mod binlog;
use crate::common::init::init_with_error;
mod schedule;
mod service;
use tracing_subscriber::Layer;
mod util;
mod vojo;
use axum::routing::get;
use axum::routing::post;
use axum::Router;

use crate::init_redis::init_redis;
use clap::Parser;
use snowflake::SnowflakeIdGenerator;
use sqlx::mysql::MySqlConnection;
use sqlx::mysql::MySqlPoolOptions;
use sqlx::{Connection, Row};
use std::vec;
use tracing_appender::non_blocking::NonBlockingBuilder;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_appender::rolling;
use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
#[macro_use]
extern crate tracing;
#[macro_use]
extern crate anyhow;
#[derive(Parser)]
#[command(author, version, about, long_about)]
struct Cli {
    /// The http port,default port is 80
    #[arg(
        default_value_t = 1000,
        short = 'C',
        long = "count",
        value_name = "Count"
    )]
    count: u32,
}
async fn create_data() -> Result<(), anyhow::Error> {
    let _work_guard = setup_logger()?;
    let cli: Cli = Cli::parse();
    let count = cli.count;
    let pool = MySqlPoolOptions::new()
        .max_connections(5)
        .connect("mysql://root:root@localhost:9306/mydb")
        .await?;
    let mut id_generator_generator = SnowflakeIdGenerator::new(1, 1);
    for i in 0..count {
        let i_str = i.to_string();
        let i_next_str = (i + 1).to_string();
        let id = id_generator_generator.real_time_generate();

        // sqlx::query("insert into user(username,first_name,content)values(?,?,?)")
        //     .bind(i_str.clone())
        //     .bind(i_str.clone())
        //     .bind(i_str.clone())
        //     .execute(&pool)
        //     .await?;
        // sqlx::query("update user set first_name = ? where username=?")
        //     .bind(i_next_str)
        //     .bind(i_str.clone())
        //     .execute(&pool)
        //     .await?;
        // sqlx::query("delete from user").execute(&pool).await?;
        sqlx::query(
            "INSERT INTO `all_types_table` (
    `tiny_int_col`,
    `small_int_col`,
    `medium_int_col`,
    `big_int_col`,
    `decimal_col`,
    `float_col`,
    `double_col`,
    `bit_col`,
    `date_col`,
    `datetime_col`,
    `time_col`,
    `year_col`,
    `char_col`,
    `varchar_col`,
    `binary_col`,
    `varbinary_col`,
    `blob_col`,
    `text_col`,
    `json_col`
) VALUES (
    ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?
)",
        )
        .bind(i % 8)
        .bind(i)
        .bind(i)
        .bind(i)
        .bind(i)
        .bind(i)
        .bind(i)
        .bind(i % 2)
        .bind("2024-08-21")
        .bind("2024-08-21 12:34:56")
        .bind("12:34:56")
        .bind(2024)
        .bind("A")
        .bind(i_str.clone())
        .bind(i_str.clone())
        .bind(i_str.clone())
        .bind(i_str.clone())
        .bind(i_str)
        .bind(r#"{"key1": "value1", "key2": "value2"}"#)
        .execute(&pool)
        .await?;
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
        .with_line_number(true)
        .with_ansi(false)
        .with_writer(non_blocking_appender)
        .with_filter(tracing_subscriber::filter::LevelFilter::INFO);
    let console_layer = FmtLayer::new()
        .with_target(true)
        .with_line_number(true)
        .with_ansi(true)
        .with_filter(tracing_subscriber::filter::LevelFilter::INFO);
    tracing_subscriber::registry()
        .with(file_layer)
        .with(console_layer)
        .with(tracing_subscriber::filter::LevelFilter::TRACE)
        .init();
    Ok(guard)
}
#[tokio::main]
async fn main() {
    if let Err(e) = main_with_error().await {
        error!("{:?}", e);
    }
}
async fn main_with_error() -> Result<(), anyhow::Error> {
    let _work_guard = setup_logger()?;
    let db_pool = common::sql_connections::create_pool().await?;
    init_with_error(db_pool.clone()).await?;
    let cloned_db_pool = db_pool.clone();
    let redis_client = init_redis().await?;
    tokio::spawn(async move {
        record_error!(main_sync_redis_loop_with_error(redis_client, cloned_db_pool).await);
    });

    let app = Router::new()
        .route(
            "/datasource",
            post(create_datasource).get(get_datasource_list),
        )
        .route("/datasource/:id/database/:name/tables", get(get_table_list))
        .route(
            "/datasource/:id",
            get(get_database_list).delete(delete_datasource_by_id),
        )
        .route("/task", post(create_task).get(get_task_list))
        .route("/auditTask", post(create_audit_task).get(get_audit_tasks))
        .route("/auditTask/execute", post(execute_audit_task))
        .route("/auditTaskResult", get(get_audit_tasks_result))
        .with_state(db_pool);
    let final_route = Router::new().nest("/api", app);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:9394").await.unwrap();
    axum::serve(listener, final_route).await.unwrap();
    Ok(())
}
