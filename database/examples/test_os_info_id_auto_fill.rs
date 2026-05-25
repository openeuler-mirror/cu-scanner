//! 测试os_info_id自动填充功能的示例程序

use database::{
    DatabaseConfig, DatabaseManager, OvalDefinition, RpmInfoObject, RpmInfoState, RpmInfoTest,
};
use utils::config::AppConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("测试os_info_id自动填充功能");

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
    todo!();
}
