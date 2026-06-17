//! 基于数据库的ID生成器演示程序

use database::{DatabaseConfig, DatabaseIdGenerator, DatabaseManager};
use std::sync::Arc;
use tokio::sync::Mutex;
use utils::config::AppConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("基于数据库的ID生成器演示程序");

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

    // 创建基于数据库的ID生成器
    let mut id_generator =
        DatabaseIdGenerator::new(db_manager.clone(), "demo_generator".to_string(), 10000);
    todo!();
}
