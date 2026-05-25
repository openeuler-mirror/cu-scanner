//! 测试列出OVAL定义的示例程序

use database::{DatabaseConfig, DatabaseManager};
use utils::config::AppConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("测试列出OVAL定义示例程序");

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

    // 查询所有OVAL定义
    let definitions = db_manager.list_all_oval_definitions().await?;

    println!("数据库中共有 {} 个OVAL定义:", definitions.len());
    for (i, definition) in definitions.iter().enumerate() {
        println!("{}. {}", i + 1, definition.id);
    }

    Ok(())
}
