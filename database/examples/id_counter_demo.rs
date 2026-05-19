//! 持久化ID计数器演示程序

use database::{DatabaseConfig, DatabaseManager, PersistentIdCounter};
use std::sync::Arc;
use tokio::sync::Mutex;
use utils::config::AppConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("持久化ID计数器演示程序");

    // 从配置文件加载数据库配置
    let config = AppConfig::from_file("config/cu-scanner.toml")?;
    let db_config = DatabaseConfig::new(
        &config.database.host,
        config.database.port,
        &config.database.database,
        &config.database.username,
        &config.database.password,
    );

    // 创建数据库管理器
    let db_manager = Arc::new(Mutex::new(DatabaseManager::new(&db_config).await?));

    // 初始化数据库表
    {
        let mut db = db_manager.lock().await;
        db.init_tables().await?;
    }

    // 创建持久化ID计数器
    let mut id_counter =
        PersistentIdCounter::new(db_manager.clone(), "demo_counter".to_string(), 10000);

    // 获取当前计数器值
    let current_counter = id_counter.get_current_counter().await?;
    todo!();
}
