CREATE TABLE sql_logs (
    id UUID DEFAULT generateUUIDv4 (),
    sync_task_id UInt32 DEFAULT 0,
    query String DEFAULT '',
    result String DEFAULT '',
    execution_time UInt64 DEFAULT 0,
    client_ip String DEFAULT '0.0.0.0',
    sql_timestamp DateTime(3) DEFAULT now64 (),
    timestamp DateTime(3) DEFAULT now64 (),
    INDEX query_text_idx query TYPE ngrambf_v1(20, 307200, 2, 0) GRANULARITY 1
) ENGINE = MergeTree()
ORDER BY (sync_task_id, sql_timestamp)
SETTINGS index_granularity = 8192;
