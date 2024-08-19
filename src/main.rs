use anyhow::anyhow;

use binlog::binlog_realtime::test_binlog_with_realtime;
use common::init_redis;
use futures::StreamExt;
use moka::future::Cache;
use mysql_async::binlog::events::{RowsEventData, RowsEventRows, TableMapEvent};
use mysql_async::binlog::value::BinlogValue;
use schedule::sync_redis::main_sync_redis_loop_with_error;
use service::audit_service::create_audit_task;
use service::database_service::get_database_list;
use service::datasource_service::{create_datasource, get_datasource_list};
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
async fn create_data() -> Result<(), anyhow::Error> {
    let _work_guard = setup_logger()?;

    let pool = MySqlPoolOptions::new()
        .max_connections(5)
        .connect("mysql://root:root@localhost:9306/mydb")
        .await?;
    let mut id_generator_generator = SnowflakeIdGenerator::new(1, 1);
    for i in 0..1000 {
        let i_str = i.to_string();
        let i_next_str = (i + 1).to_string();
        let id = id_generator_generator.real_time_generate();

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
        // sqlx::query("delete from user").execute(&pool).await?;
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
        .route("/datasource/:id", get(get_database_list))
        .route("/task", post(create_task).get(get_task_list))
        .route("/audit", post(create_audit_task))
        .with_state(db_pool);
    let final_route = Router::new().nest("/api", app);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:9394").await.unwrap();
    axum::serve(listener, final_route).await.unwrap();
    Ok(())
}
