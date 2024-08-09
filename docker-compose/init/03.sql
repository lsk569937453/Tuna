CREATE TABLE IF NOT EXISTS task (
    id INT PRIMARY KEY AUTO_INCREMENT,
    task_name VARCHAR(255) NOT NULL DEFAULT "",
    from_datasource_id INT NOT NULL DEFAULT 0,
    to_datasource_id INT NOT NULL DEFAULT 0,
    source_database_name VARCHAR(255) NOT NULL DEFAULT "",
    destination_database_name VARCHAR(255) NOT NULL DEFAULT "",
    table_mapping TEXT NOT NULL,
    timestamp TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);