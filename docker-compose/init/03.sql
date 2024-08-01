CREATE TABLE IF NOT EXISTS task (
    id INT PRIMARY KEY AUTO_INCREMENT,
    task_name VARCHAR(255) NOT NULL,
    from_datasource_id INT NOT NULL,
    to_datasource_id INT NOT NULL,
    source_database_name VARCHAR(255) NOT NULL,
    destination_database_name VARCHAR(255) NOT NULL,
    source_table_name VARCHAR(255) NOT NULL,
    destination_table_name VARCHAR(255) NOT NULL,
    syc_info VARCHAR(255) NOT NULL,
    status INT NOT NULL,
    timestamp TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE INDEX task_name_index(task)
);
