CREATE USER canal IDENTIFIED BY 'canal';

GRANT
SELECT
,
    REPLICATION SLAVE,
    REPLICATION CLIENT ON *.* TO 'canal' @'%';
-- GRANT ALL PRIVILEGES ON *.* TO 'canal'@'%' ;
FLUSH PRIVILEGES;

use mydb2;

CREATE TABLE IF NOT EXISTS `user2` (
    `id` int(11) NOT NULL AUTO_INCREMENT,
    `username` varchar(255) DEFAULT NULL,
    `first_name` varchar(50) DEFAULT NULL,
    `content` text DEFAULT NULL,
    `status` tinyint(10) DEFAULT NULL,
    PRIMARY KEY (`id`)
) ENGINE = InnoDB;
CREATE TABLE IF NOT EXISTS `all_types_table` (
    -- Numeric Types
    `id` INT(11) NOT NULL AUTO_INCREMENT,  -- Integer with auto-increment
    `tiny_int_col` TINYINT(4) DEFAULT NULL,  -- Tiny integer
    `small_int_col` SMALLINT(6) DEFAULT NULL,  -- Small integer
    `medium_int_col` MEDIUMINT(9) DEFAULT NULL,  -- Medium integer
    `big_int_col` BIGINT(20) DEFAULT NULL,  -- Big integer
    `decimal_col` DECIMAL(10,2) DEFAULT NULL,  -- Fixed-point number
    `float_col` FLOAT DEFAULT NULL,  -- Single-precision floating-point
    `double_col` DOUBLE DEFAULT NULL,  -- Double-precision floating-point
    `bit_col` BIT(8) DEFAULT NULL,  -- Bit-field
    `date_col` DATE DEFAULT NULL,  -- Date
    `datetime_col` DATETIME DEFAULT NULL,  -- Date and time
    `timestamp_col` TIMESTAMP DEFAULT CURRENT_TIMESTAMP,  -- Timestamp
    `time_col` TIME DEFAULT NULL,  -- Time
    `year_col` YEAR DEFAULT NULL,  -- Year
    `char_col` CHAR(255) DEFAULT NULL,  -- Fixed-length string
    `varchar_col` VARCHAR(255) DEFAULT NULL,  -- Variable-length string
    `binary_col` BINARY(255) DEFAULT NULL,  -- Fixed-length binary data
    `varbinary_col` VARBINARY(255) DEFAULT NULL,  -- Variable-length binary data
    `blob_col` BLOB DEFAULT NULL,  -- Binary large object
    `text_col` TEXT DEFAULT NULL,  -- Large string object
    `json_col` JSON DEFAULT NULL,
     PRIMARY KEY (`id`)
)ENGINE = InnoDB;
CREATE TABLE IF NOT EXISTS `post` (
    `post_id` int(11) NOT NULL AUTO_INCREMENT,
    `tile` varchar(255) DEFAULT NULL,
    `content` varchar(50) DEFAULT NULL,
    `status` tinyint(10) DEFAULT NULL,
    PRIMARY KEY (`post_id`)
) ENGINE = InnoDB DEFAULT CHARSET = latin1 AUTO_INCREMENT = 10001;