//! 从数据库导出OVAL定义为XML文件的示例程序

use database::{DatabaseConfig, DatabaseManager};
use std::env;
use std::fs::File;
use std::io::Write;
use utils::config::AppConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("从数据库导出OVAL定义为XML文件示例程序");

    // 获取命令行参数
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("使用方法: {} <definition_id> [output_file]", args[0]);
        std::process::exit(1);
    }

    let definition_id = &args[1];
    let output_file = if args.len() > 2 {
        args[2].clone()
    } else {
        format!("{}.xml", definition_id.replace(":", "_").replace("/", "_"))
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

    // 从数据库获取OVAL定义并转换为XML
    match db_manager.get_oval_xml_by_id(definition_id).await {
        Ok(Some(xml_content)) => {
            println!("成功获取OVAL XML内容，长度: {} 字节", xml_content.len());

            // 保存到文件
            let mut file = File::create(&output_file)?;
            file.write_all(xml_content.as_bytes())?;

            println!("OVAL XML已保存到文件: {}", output_file);
        }
        Ok(None) => {
            println!("未找到指定ID的OVAL定义");
        }
        Err(e) => {
            eprintln!("导出OVAL定义失败: {:?}", e);
        }
    }

    Ok(())
}
