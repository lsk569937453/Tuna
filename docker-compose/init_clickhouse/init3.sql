CREATE TABLE sync_task_running_logs (
    id UUID,
    sync_task_uuid UUID,
    level String,
    message String,
    sync_task_id UInt32,
    timestamp DateTime(3) DEFAULT now()
) ENGINE = MergeTree ()
ORDER BY timestamp