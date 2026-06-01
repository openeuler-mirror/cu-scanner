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

    println!("\n=== 测试1: 创建不带os_info_id的OVAL定义 ===");

    // 创建一个测试用的OVAL定义（os_info_id为None）
    let definition = OvalDefinition {
        id: "oval:test:def:1001".to_string(),
        class: "patch".to_string(),
        version: 1,
        title: "Test Definition for OS Info ID Auto Fill".to_string(),
        description: "Testing automatic os_info_id population".to_string(),
        family: "unix".to_string(),
        platform: "openEuler 20.03 LTS".to_string(),
        severity: "Important".to_string(),
        rights: "Copyright".to_string(),
        from: "test@example.com".to_string(),
        issued_date: "2025-01-01".to_string(),
        updated_date: "2025-01-01".to_string(),
        os_info_id: None, // 故意设置为None，测试自动填充
    };

    println!(
        "创建的定义: ID={}, os_info_id={:?}",
        definition.id, definition.os_info_id
    );

    // 创建包含oe1标识的RPM对象（应该匹配到openEuler 20.03）
    let rpminfo_objects = vec![RpmInfoObject {
        id: None,
        object_id: "oval:test:obj:1001".to_string(),
        ver: 1,
        rpm_name: "test-package".to_string(), // 包名不包含dist
    }];

    println!("RPM对象: rpm_name={}", rpminfo_objects[0].rpm_name);

    let references = vec![];
    let cves = vec![];
    todo!();
}
