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
    todo!();
}
