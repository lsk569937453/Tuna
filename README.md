# README
## mysqlbinlog  vs mysqldump

# 是否启用binlog日志
show variables like '%log_bin%';

# 查看详细的日志配置信息
show global variables like '%log%';

# mysql数据存储目录
show variables like '%dir%';

# 查看binlog的目录
show global variables like "%log_bin%";

# 查看当前服务器使用的biglog文件及大小
show binary logs;

# 查看最新一个binlog日志文件名称和Position
show master status;

# 事件查询命令
# IN 'log_name' ：指定要查询的binlog文件名(不指定就是第一个binlog文件)
# FROM pos ：指定从哪个pos起始点开始查起(不指定就是从整个文件首个pos点开始算)
# LIMIT [offset,] ：偏移量(不指定就是0)
# row_count ：查询总条数(不指定就是所有行)
show binlog events [IN 'log_name'] [FROM pos] [LIMIT [offset,] row_count];

# 查看 binlog 内容
show binlog events;

# 查看具体一个binlog文件的内容 （in 后面为binlog的文件名）
show binlog events in 'master.000003';

SHOW DATABASES                                //列出 MySQL Server 数据库。
SHOW TABLES [FROM db_name]                    //列出数据库数据表。
SHOW CREATE TABLES tbl_name                    //导出数据表结构。
SHOW TABLE STATUS [FROM db_name]              //列出数据表及表状态信息。
SHOW COLUMNS FROM tbl_name [FROM db_name]     //列出资料表字段
SHOW FIELDS FROM tbl_name [FROM db_name]，DESCRIBE tbl_name [col_name]。
SHOW FULL COLUMNS FROM tbl_name [FROM db_name]//列出字段及详情
SHOW FULL FIELDS FROM tbl_name [FROM db_name] //列出字段完整属性
SHOW INDEX FROM tbl_name [FROM db_name]       //列出表索引。
SHOW STATUS                                  //列出 DB Server 状态。
SHOW VARIABLES                               //列出 MySQL 系统环境变量。
SHOW PROCESSLIST                             //列出执行命令。
SHOW GRANTS FOR user                         //列出某用户权限

#redis-key:
- 任务列表:     key:tuna:task:task_id  value是状态
- 任务锁：      key:tuna:task_lock:task_id

# Command
To insert data:
```
./myapp i --count 100

```
To show the binlog:

```
./myapp s

```

```
export APP_CLICKHOUSE__URL="http://localhost:8123"
export APP_CLICKHOUSE__USER="clickhouse-user"
export APP_CLICKHOUSE__PASSWORD="secret"
export APP_CLICKHOUSE__DATABASE="tuna"

export APP_MYSQL__URL="http://localhost:3306"
export APP_MYSQL__USER="root"
export APP_MYSQL__PASSWORD="secret"
export APP_MYSQL__DATABASE="tuna"

export APP_REDIS_URL__URL="http://localhost:6379"
export APP_REDIS_URL__PASSWORD="secret"

`APP_REDIS_URL__URL="http://localhost:3333 ./target/app` would set the `APP_REDIS_URL__URL` key
```

```
SELECT sync_task_id, sync_task_uuid
FROM sync_task_running_logs
WHERE sync_task_id IN (1, 2)
  AND timestamp = (
    SELECT MAX(timestamp)
    FROM sync_task_running_logs AS sub
    WHERE sub.sync_task_id = sync_task_running_logs.sync_task_id
)

```
```
SELECT *
FROM sync_task_running_logs
WHERE
    sync_task_uuid IN (8df66dfe-6aed-d4e4-8c68-a28ab5114f7e)
    AND timestamp = (
        SELECT MIN(timestamp)
        FROM
            sync_task_running_logs AS subquery
        WHERE
            subquery.sync_task_uuid = sync_task_running_logs.sync_task_uuid
    )
ORDER BY timestamp ASC;
```