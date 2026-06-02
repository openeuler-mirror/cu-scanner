//! 数据库配置模块
//!
//! 该模块提供了数据库连接配置相关的功能实现。

use log::{error, info};
use tokio_postgres::{Client, Error as PgError};

/// 数据库错误类型
#[derive(Debug)]
pub enum DatabaseError {
    ConnectionError(PgError),
    QueryError(PgError),
    SerializationError(serde_json::Error),
}

impl std::fmt::Display for DatabaseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DatabaseError::ConnectionError(e) => write!(f, "数据库连接错误: {}", e),
            DatabaseError::QueryError(e) => write!(f, "查询错误: {}", e),
            DatabaseError::SerializationError(e) => write!(f, "序列化错误: {}", e),
        }
    }
}

impl std::error::Error for DatabaseError {}

impl From<PgError> for DatabaseError {
    fn from(error: PgError) -> Self {
        DatabaseError::ConnectionError(error)
    }
}

impl From<serde_json::Error> for DatabaseError {
    fn from(error: serde_json::Error) -> Self {
        DatabaseError::SerializationError(error)
    }
}

/// 数据库连接配置
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub database: String,
    pub username: String,
    pub password: String,
}

impl DatabaseConfig {
    pub fn new(host: &str, port: u16, database: &str, username: &str, password: &str) -> Self {
        Self {
            host: host.to_string(),
            port,
            database: database.to_string(),
            username: username.to_string(),
            password: password.to_string(),
        }
    }

    fn connection_string(&self) -> String {
        format!(
            "host={} port={} dbname={} user={} password={}",
            self.host, self.port, self.database, self.username, self.password
        )
    }
}

/// 数据库管理器
#[derive(Debug)]
pub struct DatabaseManager {
    pub client: Client,
}

impl DatabaseManager {
    /// 创建新的数据库管理器
    pub async fn new(config: &DatabaseConfig) -> Result<Self, DatabaseError> {
        info!("正在连接数据库 {}:{}", config.host, config.port);
        let (client, connection) =
            tokio_postgres::connect(&config.connection_string(), tokio_postgres::NoTls).await?;

        // Spawn连接处理任务
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                error!("数据库连接错误: {}", e);
            }
        });

        info!("数据库连接成功");
        Ok(DatabaseManager { client })
    }

    /// 获取指定ID计数器的当前值
    pub async fn get_id_counter(&self, counter_id: &str) -> Result<Option<u64>, DatabaseError> {
        let row = self
            .client
            .query_opt(
                "SELECT counter_value FROM id_counters WHERE id = $1",
                &[&counter_id],
            )
            .await?;

        if let Some(row) = row {
            let counter_value: i64 = row.get("counter_value");
            Ok(Some(counter_value as u64))
        } else {
            Ok(None)
        }
    }

    /// 设置指定ID计数器的值
    pub async fn set_id_counter(
        &self,
        counter_id: &str,
        counter_value: u64,
    ) -> Result<(), DatabaseError> {
        self.client.execute(
            "INSERT INTO id_counters (id, counter_value) VALUES ($1, $2)
             ON CONFLICT (id) DO UPDATE SET counter_value = EXCLUDED.counter_value, updated_at = CURRENT_TIMESTAMP",
            &[&counter_id, &(counter_value as i64)]
        ).await?;

        Ok(())
    }
}
