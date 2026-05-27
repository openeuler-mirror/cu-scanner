//! 测试OS信息查询功能的示例程序

use database::{DatabaseConfig, DatabaseManager};
use utils::config::AppConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("测试OS信息查询功能");

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

    // 测试1: 列出所有OS信息
    println!("\n=== 测试1: 列出所有OS信息 ===");
    let os_infos = db_manager.list_all_os_info().await?;
    for os_info in &os_infos {
        println!(
            "ID: {:?}, OS: {} {}, Dist: {}, Package: {}, Verify File: {}",
            os_info.id,
            os_info.os_type,
            os_info.os_version,
            os_info.dist,
            os_info.package_name,
            os_info.verify_file
        );
    }

    // 测试2: 根据dist查找OS信息
    println!("\n=== 测试2: 根据dist查找OS信息 ===");
    todo!();
}
