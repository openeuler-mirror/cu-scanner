//! 验证数据库导入结果的示例程序

use database::{DatabaseConfig, DatabaseManager};
use utils::config::AppConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("验证数据库导入结果示例程序");

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
    let db_manager = DatabaseManager::new(&db_config)
        .await
        .map_err(|e| format!("数据库连接失败: {:?}", e))?;

    // 列出所有OVAL定义
    println!("数据库中的所有OVAL定义:");
    let definitions = db_manager
        .list_all_oval_definitions()
        .await
        .map_err(|e| format!("查询OVAL定义失败: {:?}", e))?;

    for (i, definition) in definitions.iter().enumerate() {
        println!(
            "{}. ID: {}, 标题: {}",
            i + 1,
            definition.id,
            definition.title
        );

        // 获取该定义的详细信息
        if let Some((_, references, cves, tests, objects, states)) = db_manager
            .get_full_oval_definition(&definition.id)
            .await
            .map_err(|e| format!("获取OVAL定义详情失败: {:?}", e))?
        {
            println!("   引用数量: {}", references.len());
            println!("   CVE数量: {}", cves.len());
            println!("   测试数量: {}", tests.len());
            println!("   对象数量: {}", objects.len());
            println!("   状态数量: {}", states.len());
        }
    }

    println!("\n数据库导入验证完成");
    Ok(())
}
