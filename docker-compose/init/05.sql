CREATE TABLE IF NOT EXISTS audit_task (
    id INT PRIMARY KEY AUTO_INCREMENT,
    task_id INT NOT NULL DEFAULT 0,
    status INT NOT NULL DEFAULT 0 COMMENT '0: 未开始 1: 运行中 2: 执行完成',
    timestamp TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE INDEX task_id_unique_index(task_id)

);