//! 清空数据库并重新导入CSAF文件的示例程序

use csaf::CSAF;
use database::{DatabaseConfig, DatabaseManager};
use parser::{csaf_to_oval, process_csaf_id};
use utils::config::AppConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("清空数据库并重新导入CSAF文件示例程序");

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
    let mut db_manager = DatabaseManager::new(&db_config)
        .await
        .map_err(|e| format!("数据库连接失败: {:?}", e))?;

    // 清空所有数据表
    println!("正在清空所有数据表...");
    db_manager
        .clear_all_tables()
        .await
        .map_err(|e| format!("清空数据表失败: {:?}", e))?;

    // 初始化数据库表结构
    println!("正在初始化数据库表结构...");
    db_manager
        .init_tables()
        .await
        .map_err(|e| format!("数据库表初始化失败: {:?}", e))?;
    todo!();
}
