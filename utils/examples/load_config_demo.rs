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
    todo!();
}
