use binlog::binlog_realtime::test_binlog_with_realtime;
use common::access_log_layer::AccelogOnResponse;
use common::init_clickhouse::init_clickhouse;
use common::init_redis;
use std::time::Duration;
use tower_http::classify::ServerErrorsFailureClass;
use tower_http::trace::OnResponse;
use tower_http::ServiceBuilderExt;

use config::tuna_config::AppConfig;
use schedule::sync_redis::main_sync_redis_loop_with_error;
use service::audit_task_result_service::{
    get_audit_tasks_result, get_audit_tasks_result_by_audit_task_id,
};
use service::audit_task_service::{
    create_audit_task, delete_audit_task_by_id, execute_audit_task, get_audit_tasks,
};
use service::datasource_service::{
    create_datasource, delete_datasource_by_id, get_datasource_list,
    get_primary_key_by_datasource_id,
};
use tower_http::request_id::MakeRequestUuid;
use tracing::Span;
mod common;
mod dao;
use service::sql_log_service::{
    get_sql_logs_per_day, get_sql_logs_per_day_groupby_sync_task_id, get_sql_logs_per_minute,
    get_sql_logs_per_minute_groupby_sync_task_id, query_logs,
};
use service::sync_task_running_log_service::get_sync_task_running_logs_summary_group_by_sync_task_id;
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
mod config;
mod util;
mod vojo;
use axum::routing::post;
use axum::routing::{delete, get};
use axum::Router;

use crate::common::app_state::AppState;
use crate::common::make_span::RequestIdSpan;
use crate::init_redis::init_redis;
use chrono::FixedOffset;
use chrono::Utc;
use clap::Parser;
use snowflake::SnowflakeIdGenerator;
use sqlx::mysql::MySqlPoolOptions;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use tracing_appender::non_blocking::NonBlockingBuilder;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_appender::rolling::RollingFileAppender;
use tracing_appender::rolling::Rotation;
use tracing_subscriber::fmt::format::Writer;
use tracing_subscriber::fmt::time::FormatTime;
use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;
#[macro_use]
extern crate tracing;
#[macro_use]
extern crate anyhow;
#[macro_use]
extern crate serde_json;

use clap::Subcommand;

#[derive(Parser)]
#[command(name = "Tuna")]
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
    let app_config = AppConfig::load_config();
    println!("app_config:{:?}", app_config);
    let _work_guard = setup_logger(&app_config)?;

    let cli = Cli::parse();

    match cli.command.unwrap_or(Commands::Default) {
        Commands::Default => app_with_error(app_config).await,
        Commands::I { count } => create_data(count).await,
        Commands::S => test_binlog_with_realtime().await,
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
        let utc_now = Utc::now();
        let shanghai_tz = FixedOffset::east_opt(8 * 3600).expect("Unable to set timezone");
        let shanghai_now = utc_now.with_timezone(&shanghai_tz);
        write!(w, "{}", shanghai_now.format("%Y-%m-%d %H:%M:%S%.3f"))
    }
}
fn setup_logger(app_config: &AppConfig) -> Result<WorkerGuard, anyhow::Error> {
    let app_file = RollingFileAppender::builder()
        .rotation(Rotation::DAILY) // rotate log files once every hour
        .filename_prefix("application")
        .filename_suffix("log")
        .max_log_files(2)
        .build("./logs")?;
    let access_log_file = RollingFileAppender::builder()
        .rotation(Rotation::DAILY) // rotate log files once every hour
        .filename_prefix("access")
        .filename_suffix("log")
        .max_log_files(100)
        .build("./logs")?;
    let access_log_layer = tracing_subscriber::fmt::Layer::new()
        .with_writer(access_log_file)
        .with_timer(ShanghaiTime)
        .with_target(true)
        .with_ansi(false)
        .with_filter(tracing_subscriber::filter::LevelFilter::INFO)
        .with_filter(EnvFilter::new("access_log=info"));
    let (non_blocking_appender, guard) = NonBlockingBuilder::default()
        .buffered_lines_limit(10)
        .finish(app_file);
    let app_file_layer = tracing_subscriber::fmt::Layer::new()
        .with_timer(ShanghaiTime)
        .with_target(true)
        .with_line_number(true)
        .with_ansi(false)
        .with_writer(non_blocking_appender)
        .with_filter(tracing_subscriber::filter::LevelFilter::INFO)
        .with_filter(
            EnvFilter::new("info").add_directive("access_log=off".parse().unwrap()), // Allow access_log at INFO level
        );
    let console_layer = FmtLayer::new()
        .with_timer(ShanghaiTime)
        .with_target(true)
        .with_line_number(true)
        .with_ansi(true)
        .with_filter(tracing_subscriber::filter::LevelFilter::INFO);

    let subscriber = tracing_subscriber::registry()
        .with(tracing_subscriber::filter::LevelFilter::TRACE)
        .with(access_log_layer)
        .with(app_file_layer);
    let show_console = app_config
        .logging
        .clone()
        .unwrap_or_default()
        .console
        .unwrap_or(false);

    if show_console {
        subscriber.with(console_layer).init();
    } else {
        subscriber.init();
    }

    Ok(guard)
}

async fn app_with_error(app_config: AppConfig) -> Result<(), anyhow::Error> {
    let db_pool = common::sql_connections::create_pool(&app_config.mysql).await?;
    init_with_error(db_pool.clone()).await?;
    let redis_client = init_redis(&app_config.redis).await?;
    let clickhouse_client = init_clickhouse(&app_config.clickhouse).await?;

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
        .route("/datasource/:id/tables", get(get_table_list))
        .route(
            "/datasource/:id/table/:table_name",
            get(get_primary_key_by_datasource_id),
        )
        .route("/datasource/:id", delete(delete_datasource_by_id))
        .route("/syncTask", post(create_task).get(get_task_list))
        .route("/syncTask/status/:id", get(get_sync_task_status_by_id))
        .route("/syncTask/:id", delete(delete_sync_task_by_id))
        .route("/auditTask", post(create_audit_task).get(get_audit_tasks))
        .route("/auditTask/:id", delete(delete_audit_task_by_id))
        .route("/auditTask/execute", post(execute_audit_task))
        .route("/auditTaskResult", get(get_audit_tasks_result))
        .route(
            "/auditTaskResult/:id",
            get(get_audit_tasks_result_by_audit_task_id),
        )
        .route("/sqlLogs/perMinute", get(get_sql_logs_per_minute))
        .route("/sqlLogs/perDay", get(get_sql_logs_per_day))
        .route(
            "/sqlLogs/perMinuteTaskId",
            get(get_sql_logs_per_minute_groupby_sync_task_id),
        )
        .route(
            "/sqlLogs/perDayTaskId",
            get(get_sql_logs_per_day_groupby_sync_task_id),
        )
        .route("/sqlLogs", post(query_logs))
        .route(
            "/syncTaskLogs/summaryByTaskId",
            get(get_sync_task_running_logs_summary_group_by_sync_task_id),
        )
        .with_state(cloned_shared_state);

    let middleware = ServiceBuilder::new()
        .set_x_request_id(MakeRequestUuid)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(RequestIdSpan)
                .on_response(AccelogOnResponse)
                .on_failure(|_: ServerErrorsFailureClass, _latency: Duration, _span: &Span| {}),
        );
    let final_route = Router::new().nest("/api", app).layer(
        middleware, // Add access log layer here
    );
    let listener = tokio::net::TcpListener::bind("0.0.0.0:9394").await?;
    axum::serve(listener, final_route).await?;
    Ok(())
}
