//! 从数据库导出OVAL文件的示例程序

use database::{DatabaseConfig, DatabaseManager};
use std::fs;
use utils::config::AppConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("开始从数据库导出OVAL文件示例...");

    // 从配置文件加载数据库配置
    let config = AppConfig::from_file("config/cu-scanner.toml")?;
    let db_config = DatabaseConfig::new(
        &config.database.host,
        config.database.port,
        &config.database.database,
        &config.database.username,
        &config.database.password,
    );

    // 连接数据库
    let mut db_manager = DatabaseManager::new(&db_config).await?;

    // 初始化数据库表结构（如果尚未初始化）
    db_manager.init_tables().await?;

    // 获取第一个OVAL定义的ID（这里使用示例ID，您需要根据实际情况修改）
    // 在实际使用中，您可能需要先查询数据库获取有效的ID
    todo!();
}
