//! 解析CSAF文件，转换为OVAL格式并存储到数据库的示例程序

use csaf::CSAF;
use database::{DatabaseConfig, DatabaseManager};
use parser::{csaf_to_oval, process_csaf_id};
use std::env;
use utils::config::AppConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("CSAF到数据库存储示例程序");

    // 获取命令行参数
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("使用方法: {} <csaf_file_path>", args[0]);
        std::process::exit(1);
    }

    let csaf_file_path = &args[1];
    println!("正在处理CSAF文件: {}", csaf_file_path);

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
    let mut db_manager = DatabaseManager::new(&db_config)
        .await
        .map_err(|e| format!("数据库连接失败: {:?}", e))?;

    // 初始化数据库表结构
    println!("正在初始化数据库表结构...");
    db_manager
        .init_tables()
        .await
        .map_err(|e| format!("数据库表初始化失败: {:?}", e))?;

    // 加载CSAF文件
    println!("正在加载CSAF文件...");
    todo!();
}
