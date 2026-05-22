//! 处理第二个CSAF文件的示例程序

use csaf::CSAF;
use database::{DatabaseConfig, DatabaseManager};
use parser::csaf_to_oval;
use utils::config::AppConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("处理第二个CSAF文件示例程序");

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
    let mut db_manager = DatabaseManager::new(&db_manager_config).await?;

    // 加载第二个CSAF测试文件
    println!("加载第二个CSAF测试文件...");
    let csaf = CSAF::from_file(
        "/home/fatmouse/workspace/cu-scanner/test/csaf/csaf-openeuler-sa-2025-1009.json",
    )
    .map_err(|e| format!("加载CSAF文件失败: {}", e))?;
    println!("CSAF文件加载成功: {}", csaf.document.title);

    // 使用默认计数器转换CSAF到OVAL
    println!("转换CSAF到OVAL格式...");
    let oval = csaf_to_oval(&csaf).map_err(|e| format!("CSAF到OVAL转换失败: {}", e))?;
    todo!();
}
