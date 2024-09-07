use binlog::binlog_realtime::test_binlog_with_realtime;
use common::init_clickhouse::init_clickhouse;
use common::init_redis;

use schedule::sync_redis::main_sync_redis_loop_with_error;
use service::audit_task_result_service::get_audit_tasks_result;
use service::audit_task_service::{
    create_audit_task, delete_audit_task_by_id, execute_audit_task, get_audit_tasks,
};
use service::database_service::get_database_list;
use service::datasource_service::{
    create_datasource, delete_datasource_by_id, get_datasource_list,
};
mod common;
mod dao;
use service::sync_task_servivce::{
    create_task, delete_sync_task_by_id, get_sync_task_status_by_id, get_task_list,
};
use service::table_service::get_table_list;
use tracing_subscriber::fmt::Layer as FmtLayer;
mod binlog;
use crate::common::init::init_with_error;
mod schedule;
mod service;
use tracing_subscriber::Layer;
mod util;
mod vojo;
use axum::routing::post;
use axum::routing::{delete, get};
use axum::Router;

use crate::common::app_state::AppState;
use crate::init_redis::init_redis;
use chrono::FixedOffset;
use chrono::Utc;
use clap::Parser;
use snowflake::SnowflakeIdGenerator;
use sqlx::mysql::MySqlPoolOptions;
use tracing_appender::non_blocking::NonBlockingBuilder;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_appender::rolling::RollingFileAppender;
use tracing_appender::rolling::{self, Rotation};
use tracing_subscriber::fmt::format::Writer;
use tracing_subscriber::fmt::time::FormatTime;
use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
#[macro_use]
extern crate tracing;
#[macro_use]
extern crate anyhow;
#[macro_use]
extern crate serde_json;

use clap::Subcommand;

#[derive(Parser)]
#[command(name = "MyApp")]
#[command(about = "An application with multiple flags and arguments")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Start the HTTP server
    Default,

    /// Insert data into the database
    I {
        /// The number of records to insert
        #[arg(short, long)]
        count: i32,
    },

    /// Show the binlog
    S,
}
#[tokio::main]
async fn main() {
    if let Err(err) = main_with_error().await {
        error!("main_with_error error:{:?}", err);
    }
}
async fn main_with_error() -> Result<(), anyhow::Error> {
    let _work_guard = setup_logger()?;

    let cli = Cli::parse();

    match cli.command.unwrap_or(Commands::Default) {
        Commands::Default => app_with_error().await,
        Commands::I { count } => create_data(count).await,
        Commands::S => {
            test_binlog_with_realtime().await
            // Add your logic here
        }
    }
}

async fn create_data(count: i32) -> Result<(), anyhow::Error> {
    let pool = MySqlPoolOptions::new()
        .max_connections(5)
        .connect("mysql://root:root@localhost:9306/mydb")
        .await?;
    let mut id_generator_generator = SnowflakeIdGenerator::new(1, 1);
    for i in 0..count {
        let i_str = i.to_string();
        let _ = (i + 1).to_string();
        let _ = id_generator_generator.real_time_generate();

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
struct ShanghaiTime;

impl FormatTime for ShanghaiTime {
    fn format_time(&self, w: &mut Writer<'_>) -> std::fmt::Result {
        // Get the current time in UTC
        let utc_now = Utc::now();
        // Convert to Asia/Shanghai timezone (UTC+8)
        let shanghai_tz = FixedOffset::east_opt(8 * 3600).expect("Unable to set timezone");
        let shanghai_now = utc_now.with_timezone(&shanghai_tz);
        // Format the time
        write!(w, "{}", shanghai_now.format("%Y-%m-%d %H:%M:%S%.3f"))
    }
}
fn setup_logger() -> Result<WorkerGuard, anyhow::Error> {
    let app_file = RollingFileAppender::builder()
        .rotation(Rotation::DAILY) // rotate log files once every hour
        .filename_prefix("access")
        .filename_suffix("log")
        .max_log_files(2)
        .build("./logs")?;
    let (non_blocking_appender, guard) = NonBlockingBuilder::default()
        .buffered_lines_limit(10)
        .finish(app_file);
    let file_layer = tracing_subscriber::fmt::Layer::new()
        .with_timer(ShanghaiTime)
        .with_target(true)
        .with_line_number(true)
        .with_ansi(false)
        .with_writer(non_blocking_appender)
        .with_filter(tracing_subscriber::filter::LevelFilter::INFO);
    let console_layer = FmtLayer::new()
        .with_timer(ShanghaiTime)
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

async fn app_with_error() -> Result<(), anyhow::Error> {
    let db_pool = common::sql_connections::create_pool().await?;
    init_with_error(db_pool.clone()).await?;
    let redis_client = init_redis().await?;
    let clickhouse_client = init_clickhouse().await?;

    // Combine them into a single shared state
    let shared_state = AppState {
        db_pool,
        redis_client,
        clickhouse_client,
    };
    let cloned_shared_state = shared_state.clone();
    tokio::spawn(async move {
        record_error!(main_sync_redis_loop_with_error(shared_state).await);
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
        .route("/syncTask/status/:id", get(get_sync_task_status_by_id))
        .route("/syncTask/:id", delete(delete_sync_task_by_id))
        .route("/auditTask", post(create_audit_task).get(get_audit_tasks))
        .route("/auditTask/:id", delete(delete_audit_task_by_id))
        .route("/auditTask/execute", post(execute_audit_task))
        .route("/auditTaskResult", get(get_audit_tasks_result))
        .with_state(cloned_shared_state);
    let final_route = Router::new().nest("/api", app);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:9394").await?;
    axum::serve(listener, final_route).await?;
    Ok(())
}
