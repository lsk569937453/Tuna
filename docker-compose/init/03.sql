CREATE TABLE IF NOT EXISTS sync_task (
    id INT PRIMARY KEY AUTO_INCREMENT,
    task_name VARCHAR(255) NOT NULL DEFAULT "",
    from_datasource_id INT NOT NULL DEFAULT 0,
    to_datasource_id INT NOT NULL DEFAULT 0,
    from_datasource_url VARCHAR(255) NOT NULL DEFAULT "",
    to_datasource_url VARCHAR(255) NOT NULL DEFAULT "",
    from_database_name VARCHAR(255) NOT NULL DEFAULT "",
    to_database_name VARCHAR(255) NOT NULL DEFAULT "",
    table_mapping TEXT NOT NULL,
    timestamp TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE INDEX idx_union_unique (
        from_datasource_id,
        to_datasource_id,
        from_database_name,
        to_database_name
    )
);