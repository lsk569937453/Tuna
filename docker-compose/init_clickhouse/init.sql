CREATE TABLE IF NOT EXISTS tuna.audit_task_result
(
    id UInt32,
    audit_task_id UInt32 DEFAULT 0,
    left_compare String DEFAULT '',
    right_compare String DEFAULT '',
    timestamp DateTime DEFAULT now()
) 
ENGINE = MergeTree()
ORDER BY id;