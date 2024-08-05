CREATE TABLE IF NOT EXISTS task (
    id INT PRIMARY KEY AUTO_INCREMENT,
    task_name VARCHAR(255) NOT NULL,
    from_datasource_id INT NOT NULL,
    to_datasource_id INT NOT NULL,
    source_database_name VARCHAR(255) NOT NULL,
    destination_database_name VARCHAR(255) NOT NULL,
    source_table_name VARCHAR(255) NOT NULL,
    destination_table_name VARCHAR(255) NOT NULL,
    status INT NOT NULL,
    worker_ip VARCHAR(255) NOT NULL,
    offset
        VARCHAR(500) NOT NULL,
        timestamp TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);