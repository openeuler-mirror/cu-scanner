//! 根据 SA ID 从数据库创建 OVAL 定义示例程序
//!
//! 该示例程序演示了如何根据 SA ID 从 CSAF 数据库中查询信息并创建 OVAL 定义。

use csaf_database::DatabaseConfig;
use parser::csaf_db_parser::create_definition_from_sa_id;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("=== 根据 SA ID 创建 OVAL 定义示例 ===\n");

    // 配置 CSAF 数据库连接
    let csaf_db_config = DatabaseConfig::new(
        "82.156.53.132", // 数据库主机
        5432,            // 数据库端口
        "cu_cveadmin",   // 数据库名称
        "cu_cveadmin",   // 用户名
        "Ninjia1000SX",  // 密码
    );

    println!("CSAF 数据库配置:");
    println!("  主机: {}", csaf_db_config.host);
    println!("  端口: {}", csaf_db_config.port);
    println!("  数据库: {}", csaf_db_config.database);
    println!("  用户名: {}", csaf_db_config.username);
    println!();

    // 指定要查询的 SA ID
    let sa_id = "SA-2025-1004";
    println!("正在根据 SA ID '{}' 创建 OVAL 定义...\n", sa_id);

    // 从数据库查询信息并创建 OVAL 定义
    match create_definition_from_sa_id(&csaf_db_config, sa_id).await {
        Ok(definition) => {
            println!("成功创建 OVAL 定义！\n");
            println!("OVAL 定义信息:");
            println!("  ID: {}", definition.id);
            println!("  版本: {}", definition.version);
            println!("  类别: {}", definition.class);
            println!("  标题: {}", definition.metadata.title);
            println!("  描述: {}", definition.metadata.description);
            println!("  平台: {}", definition.metadata.affected.platform);
            println!("  严重性: {}", definition.metadata.advisory.severity);
            println!("  发布时间: {}", definition.metadata.advisory.issued.date);
            println!("  更新时间: {}", definition.metadata.advisory.updated.date);

            println!("\n示例执行成功！");
        }
        Err(e) => {
            eprintln!("创建 OVAL 定义失败: {}", e);
            return Err(e);
        }
    }

    Ok(())
}
