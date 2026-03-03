//! 从数据库导出OVAL文件的示例程序

use database::{DatabaseConfig, DatabaseManager};
use std::fs;
use utils::config::AppConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("开始从数据库导出OVAL文件示例...");

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

    // 初始化数据库表结构（如果尚未初始化）
    db_manager.init_tables().await?;

    // 获取第一个OVAL定义的ID（这里使用示例ID，您需要根据实际情况修改）
    // 在实际使用中，您可能需要先查询数据库获取有效的ID
    let oval_id = "oval:cn.chinaunicom.culinux.cusa:def:10001";

    println!("正在尝试导出ID为 {} 的OVAL定义...", oval_id);

    // 从数据库获取OVAL XML内容
    match db_manager.get_oval_xml_by_id(oval_id).await? {
        Some(xml_content) => {
            // 保存到文件
            let filename = format!("exported_{}.xml", oval_id.replace(":", "_"));
            fs::write(&filename, xml_content)?;
            println!("成功导出OVAL文件: {}", filename);
        }
        None => {
            println!("未找到ID为 {} 的OVAL定义", oval_id);
            println!("请确保数据库中存在该ID的OVAL定义，或者使用其他有效的ID");
        }
    }

    Ok(())
}
