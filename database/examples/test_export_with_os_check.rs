//! 测试从数据库导出OVAL XML时是否包含OS检查信息

use database::{DatabaseConfig, DatabaseManager};
use utils::config::AppConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("测试从数据库导出OVAL XML时是否包含OS检查信息\n");

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
    let db_manager = DatabaseManager::new(&db_config).await?;

    // 列出所有OVAL定义
    println!("正在查询所有OVAL定义...");
    todo!();
}
