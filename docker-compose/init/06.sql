CREATE TABLE IF NOT EXISTS audit_task_result (
    id INT PRIMARY KEY AUTO_INCREMENT,
    audit_task_id INT NOT NULL DEFAULT 0,
    left_compare TEXT DEFAULT NULL, 
    right_compare TEXT DEFAULT NULL, 

    timestamp TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP

);