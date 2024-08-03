CREATE USER canal IDENTIFIED BY 'canal';

GRANT
SELECT
,
    REPLICATION SLAVE,
    REPLICATION CLIENT ON *.* TO 'canal' @'%';
-- GRANT ALL PRIVILEGES ON *.* TO 'canal'@'%' ;
FLUSH PRIVILEGES;

use mydb;

CREATE TABLE IF NOT EXISTS `user` (
    `user_id` int(11) NOT NULL AUTO_INCREMENT,
    `username` varchar(255) DEFAULT NULL,
    `first_name` varchar(50) DEFAULT NULL,
    `content` text DEFAULT NULL,
    `status` tinyint(10) DEFAULT NULL,
    PRIMARY KEY (`user_id`)
) ENGINE = InnoDB DEFAULT CHARSET = latin1 AUTO_INCREMENT = 10001;

CREATE TABLE IF NOT EXISTS `post` (
    `post_id` int(11) NOT NULL AUTO_INCREMENT,
    `tile` varchar(255) DEFAULT NULL,
    `content` varchar(50) DEFAULT NULL,
    `status` tinyint(10) DEFAULT NULL,
    PRIMARY KEY (`post_id`)
) ENGINE = InnoDB DEFAULT CHARSET = latin1 AUTO_INCREMENT = 10001;