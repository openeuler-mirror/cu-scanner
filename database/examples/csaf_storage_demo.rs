//! CSAF到OVAL转换并存储到数据库的演示程序

use csaf::CSAF;
use database::{DatabaseConfig, DatabaseManager};
use parser::csaf_to_oval;
use utils::config::AppConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("CSAF到OVAL转换并存储到数据库演示");

    // 从配置文件加载数据库配置
    println!("加载配置文件...");
    let config =
        AppConfig::from_file("/home/fatmouse/workspace/cu-scanner/config/cu-scanner.toml")?;
    let db_config = &config.database;

    // 创建数据库管理器配置
    let db_manager_config = DatabaseConfig::new(
        &db_config.host,
        db_config.port,
        &db_config.database,
        &db_config.username,
        &db_config.password,
    );

    // 创建数据库管理器
    todo!();
}
