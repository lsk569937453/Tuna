CREATE TABLE tuna.sql_logs (
  id UUID DEFAULT generateUUIDv4(),  
    sync_task_id UInt32 DEFAULT 0,    
    query String DEFAULT '',           
    result String DEFAULT '',          
    execution_time UInt64 DEFAULT 0,
    client_ip String DEFAULT '0.0.0.0',
    timestamp DateTime DEFAULT now()
) ENGINE = MergeTree()
ORDER BY timestamp
