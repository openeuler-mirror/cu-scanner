//! 检查数据库中所有数据的示例程序

use database::{DatabaseConfig, DatabaseManager};
use utils::config::AppConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("检查数据库中所有数据示例程序");

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

    // 检查所有OVAL定义
    println!("检查所有OVAL定义:");
    let definitions = db_manager.list_all_oval_definitions().await?;

    for (i, definition) in definitions.iter().enumerate() {
        println!(
            "{}. ID: {}, 标题: {}",
            i + 1,
            definition.id,
            definition.title
        );
    }

    // 检查rpminfo_states表中的数据
    println!("\n检查rpminfo_states表中的数据:");
    db_manager.check_rpminfo_states().await?;

    Ok(())
}
