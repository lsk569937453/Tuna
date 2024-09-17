CREATE TABLE sql_logs (
    id UUID DEFAULT generateUUIDv4 (),
    sync_task_id UInt32 DEFAULT 0,
    query String DEFAULT '',
    result String DEFAULT '',
    execution_time UInt64 DEFAULT 0,
    client_ip String DEFAULT '0.0.0.0',
    sql_timestamp DateTime(3) DEFAULT now64 (),
    timestamp DateTime(3) DEFAULT now64 ()
) ENGINE = MergeTree ()
ORDER BY timestamp