CREATE TABLE my_db_name.logs (
    timestamp DateTime,
    level String,
    message String,
    service_name String,
    file String,
    line UInt32
) ENGINE = MergeTree()
ORDER BY (timestamp, level);
