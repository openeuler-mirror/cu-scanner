//! 验证数据库导入结果的示例程序

use database::{DatabaseConfig, DatabaseManager};
use utils::config::AppConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("验证数据库导入结果示例程序");

    // 从配置文件加载数据库配置
    let config = AppConfig::from_file("config/cu-scanner.toml")
        .map_err(|e| format!("配置文件加载失败: {}", e))?;
    let db_config = DatabaseConfig::new(
        &config.database.host,
        config.database.port,
        &config.database.database,
        &config.database.username,
        &config.database.password,
    );

    // 连接数据库
    let db_manager = DatabaseManager::new(&db_config)
        .await
        .map_err(|e| format!("数据库连接失败: {:?}", e))?;

    // 列出所有OVAL定义
    println!("数据库中的所有OVAL定义:");
    todo!();
}
