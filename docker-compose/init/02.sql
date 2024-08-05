CREATE TABLE IF NOT EXISTS datasource (
    id INT PRIMARY KEY AUTO_INCREMENT,
    datasource_name VARCHAR(255) NOT NULL,
    datasource_url VARCHAR(255) NOT NULL,
    host VARCHAR(255) NOT NULL,
    port INT NOT NULL DEFAULT 0,
    timestamp TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE INDEX datasource_url_unique_index(datasource_url)
);
