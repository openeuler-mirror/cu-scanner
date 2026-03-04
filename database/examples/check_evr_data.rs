//! 检查rpminfo_states表数据的示例程序（包含EVR信息）

use database::{DatabaseConfig, DatabaseManager};
use utils::config::AppConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("检查rpminfo_states表数据示例程序（包含EVR信息）");

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

    // 检查rpminfo_states表中的数据（包含EVR信息）
    db_manager.check_rpminfo_states().await?;

    Ok(())
}
