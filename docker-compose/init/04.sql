CREATE TABLE IF NOT EXISTS task_running_info (
    id INT PRIMARY KEY AUTO_INCREMENT,
    task_id VARCHAR(255) NOT NULL,
    status INT NOT NULL,
    worker_ip VARCHAR(255) NOT NULL,
    offset  VARCHAR(500) NOT NULL,
    timestamp TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE INDEX task_name_index(task)
);
