CREATE TABLE IF NOT EXISTS task (
    id INT PRIMARY KEY AUTO_INCREMENT,
    task_name VARCHAR(255) NOT NULL DEFAULT "",
    from_datasource_id INT NOT NULL DEFAULT 0,
    to_datasource_id INT NOT NULL DEFAULT 0,
    source_database_name VARCHAR(255) NOT NULL DEFAULT "",
    destination_database_name VARCHAR(255) NOT NULL DEFAULT "",
    source_table_name VARCHAR(255) NOT NULL DEFAULT "",
    destination_table_name VARCHAR(255) NOT NULL DEFAULT "",
    status INT NOT NULL DEFAULT 0 COMMENT '0: 未开始 1: 运行中 2: 执行成功 3: 执行失败',
    worker_ip VARCHAR(255) NOT NULL DEFAULT "",
    binlog_name VARCHAR(255) NOT NULL DEFAULT "",
    offset
        VARCHAR(500) NOT NULL DEFAULT "",
        timestamp TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);