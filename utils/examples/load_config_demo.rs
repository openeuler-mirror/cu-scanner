//! 配置文件加载示例程序
//!
//! 该示例程序演示了如何从 cu-scanner.toml 文件加载配置。

use utils::config::AppConfig;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== 配置文件加载示例 ===\n");

    // 从配置文件加载配置
    let config_path = "config/cu-scanner.toml";
    println!("正在加载配置文件: {}", config_path);

    let config = AppConfig::from_file(config_path)?;

    // 显示主数据库配置
    println!("\n【主数据库配置】");
    println!("  主机: {}", config.database.host);
    println!("  端口: {}", config.database.port);
    println!("  数据库: {}", config.database.database);
    println!("  用户名: {}", config.database.username);
    println!("  密码: {}", "*".repeat(config.database.password.len()));

    // 显示 CSAF 数据库配置
    if let Some(ref csaf_db) = config.csaf_db {
        println!("\n【CSAF数据库配置】");
        println!("  主机: {}", csaf_db.host);
        println!("  端口: {}", csaf_db.port);
        println!("  数据库: {}", csaf_db.database);
        println!("  用户名: {}", csaf_db.username);
        println!("  密码: {}", "*".repeat(csaf_db.password.len()));
    } else {
        println!("\n【CSAF数据库配置】");
        println!("  未配置");
    }

    // 显示日志配置
    println!("\n【日志配置】");
    println!("  日志级别: {}", config.logging.level);
    println!(
        "  日志文件: {}",
        if config.logging.file.is_empty() {
            "未配置"
        } else {
            &config.logging.file
        }
    );
    println!("  标准输出: {}", config.logging.stdout);

    // 显示 API 配置
    println!("\n【API配置】");
    println!("  分组名称: {}", config.api.group_name);

    println!("\n配置加载成功！");
    Ok(())
}
