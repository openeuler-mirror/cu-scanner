//! 数据库配置模块
//!
//! 该模块提供了数据库连接配置相关的功能实现。

use log::{error, info};
use tokio_postgres::{Client, Error as PgError};

/// 数据库错误类型
#[derive(Debug, thiserror::Error)]
pub enum DatabaseError {
    #[error("数据库连接错误: {0}")]
    ConnectionError(#[from] PgError),
    #[error("序列化错误: {0}")]
    SerializationError(#[from] serde_json::Error),
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
    /// 创建新的数据库配置
    pub fn new(host: &str, port: u16, database: &str, username: &str, password: &str) -> Self {
        Self {
            host: host.to_string(),
            port,
            database: database.to_string(),
            username: username.to_string(),
            password: password.to_string(),
        }
    }

    /// 生成连接字符串
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
}
