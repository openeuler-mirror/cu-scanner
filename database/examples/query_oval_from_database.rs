//! 从数据库查询OVAL定义的示例程序

use database::{DatabaseConfig, DatabaseManager};
use std::env;
use utils::config::AppConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("从数据库查询OVAL定义示例程序");

    // 获取命令行参数
    let args: Vec<String> = env::args().collect();
    let definition_id = if args.len() > 1 {
        args[1].clone()
    } else {
        "oval:cn.chinaunicom.culinux.cusa:def:20251009".to_string()
    };

    println!("正在查询OVAL定义: {}", definition_id);

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

    // 查询OVAL定义
    match db_manager.get_full_oval_definition(&definition_id).await {
        Ok(Some((definition, references, cves, tests, objects, states))) => {
            println!("成功查询到OVAL定义:");
            println!("  ID: {}", definition.id);
            println!("  标题: {}", definition.title);
            println!("  描述: {}", definition.description);
            println!("  平台: {}", definition.platform);
            println!("  严重性: {}", definition.severity);
            println!("  引用数量: {}", references.len());
            println!("  CVE数量: {}", cves.len());
            println!("  测试数量: {}", tests.len());
            println!("  对象数量: {}", objects.len());
            println!("  状态数量: {}", states.len());

            // 打印引用信息
            if !references.is_empty() {
                println!("  引用:");
                for reference in &references {
                    println!("    - {}: {}", reference.ref_id, reference.ref_url);
                }
            }

            // 打印CVE信息
            if !cves.is_empty() {
                println!("  CVE:");
                for cve in &cves {
                    println!("    - {}: {}", cve.cve_id, cve.impact);
                }
            }
        }
        Ok(None) => {
            println!("未找到指定ID的OVAL定义");
        }
        Err(e) => {
            eprintln!("查询OVAL定义失败: {:?}", e);
        }
    }

    Ok(())
}
