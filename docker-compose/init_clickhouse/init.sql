CREATE TABLE IF NOT EXISTS tuna.audit_task_result (
    id UUID,
    audit_task_id UInt32 DEFAULT 0,
    execution_id String DEFAULT '',
    primary_id String DEFAULT '',
    left_compare String DEFAULT '',
    right_compare String DEFAULT '',
    is_same UInt32 DEFAULT 0,
    timestamp DateTime DEFAULT now()
) ENGINE = ReplacingMergeTree ()
ORDER BY (execution_id, primary_id);