CREATE TABLE tuna.sql_logs (
    id UUID,
    query String,
    result String,
    execution_time UInt64,
    timestamp DateTime DEFAULT now()
) ENGINE = MergeTree()
ORDER BY timestamp
