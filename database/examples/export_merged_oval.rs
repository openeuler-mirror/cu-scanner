//! 批量导出并合并多个OVAL定义为单个XML文件的示例程序

use database::{DatabaseConfig, DatabaseManager};
use std::env;
use std::fs::File;
use std::io::Write;
use utils::config::AppConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("批量导出并合并OVAL定义示例程序");

    // 获取命令行参数
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_usage(&args[0]);
        std::process::exit(1);
    }

    // 加载数据库配置
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

    let mode = &args[1];

    match mode.as_str() {
        "--all" => {
            // 导出所有定义
            println!("正在导出所有OVAL定义...");
            let merged = db_manager.export_all_oval_definitions().await?;

            let output_file = if args.len() > 2 {
                args[2].clone()
            } else {
                "all_oval_definitions.xml".to_string()
            };

            save_oval(&merged, &output_file)
                .map_err(|e| -> Box<dyn std::error::Error> { Box::new(std::io::Error::new(std::io::ErrorKind::Other, e)) })?;
        }
        "--ids" => {
            // 导出指定ID列表
            if args.len() < 3 {
                eprintln!("错误: --ids 模式需要至少一个definition ID");
                print_usage(&args[0]);
                std::process::exit(1);
            }

            let definition_ids: Vec<String> = args[2..args.len()-1].iter().map(|s| s.clone()).collect();
            let output_file = args.last().unwrap();

            println!("正在导出 {} 个OVAL定义...", definition_ids.len());
            let merged = db_manager.export_merged_oval(definition_ids).await?;

            save_oval(&merged, output_file)
                .map_err(|e| -> Box<dyn std::error::Error> { Box::new(std::io::Error::new(std::io::ErrorKind::Other, e)) })?;
        }
        "--date-range" => {
            // 导出指定时间范围
            if args.len() < 5 {
                eprintln!("错误: --date-range 模式需要 start_date end_date output_file");
                eprintln!("示例: {} --date-range 2025-01-01 2025-01-31 output.xml", args[0]);
                std::process::exit(1);
            }

            let start_date = &args[2];
            let end_date = &args[3];
            let output_file = &args[4];

            println!("正在导出 {} 到 {} 之间的OVAL定义...", start_date, end_date);
            let merged = db_manager.export_oval_by_date_range(start_date, end_date).await?;

            save_oval(&merged, output_file)
                .map_err(|e| -> Box<dyn std::error::Error> { Box::new(std::io::Error::new(std::io::ErrorKind::Other, e)) })?;
        }
        _ => {
            eprintln!("错误: 未知的模式 '{}'", mode);
            print_usage(&args[0]);
            std::process::exit(1);
        }
    }

    Ok(())
}

/// 保存OVAL定义到文件
fn save_oval(oval: &oval::OvalDefinitions, output_file: &str) -> Result<(), String> {
    println!("合并后的OVAL统计信息:");
    println!("  - Definitions: {}", oval.get_definition_count());
    println!("  - Tests: {}", oval.get_test_count());
    println!("  - Objects: {}", oval.get_object_count());
    println!("  - States: {}", oval.get_state_count());

    let xml = oval.to_oval_string()
        .map_err(|e| format!("转换为XML失败: {}", e))?;
    let mut file = File::create(output_file)
        .map_err(|e| format!("创建文件失败: {}", e))?;
    file.write_all(xml.as_bytes())
        .map_err(|e| format!("写入文件失败: {}", e))?;

    println!("✓ OVAL XML已保存到文件: {}", output_file);
    println!("  文件大小: {} 字节", xml.len());

    Ok(())
}

/// 打印使用说明
fn print_usage(program: &str) {
    println!("使用方法:");
    println!();
    println!("1. 导出所有OVAL定义:");
    println!("   {} --all [output_file]", program);
    println!("   示例: {} --all all_definitions.xml", program);
    println!();
    println!("2. 导出指定ID列表:");
    println!("   {} --ids <id1> <id2> ... <output_file>", program);
    println!("   示例: {} --ids oval:cn.chinaunicom.culinux.cusa:def:20251004 oval:cn.chinaunicom.culinux.cusa:def:20251009 merged.xml", program);
    println!();
    println!("3. 导出指定时间范围:");
    println!("   {} --date-range <start_date> <end_date> <output_file>", program);
    println!("   示例: {} --date-range 2025-01-01 2025-01-31 january.xml", program);
}
