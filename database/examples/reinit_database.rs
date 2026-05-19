//! 重新初始化数据库表结构的示例程序

use database::{DatabaseConfig, DatabaseManager};
use utils::config::AppConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("重新初始化数据库表结构示例程序");

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

    // 重新初始化表结构
    db_manager.reinit_tables().await?;

    println!("数据库表结构重新初始化完成");

    Ok(())
}
