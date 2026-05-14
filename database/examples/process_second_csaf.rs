//! 处理第二个CSAF文件的示例程序

use csaf::CSAF;
use database::{DatabaseConfig, DatabaseManager};
use parser::csaf_to_oval;
use utils::config::AppConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("处理第二个CSAF文件示例程序");

    // 从配置文件加载数据库配置
    println!("加载配置文件...");
    let config =
        AppConfig::from_file("/home/fatmouse/workspace/cu-scanner/config/cu-scanner.toml")?;
    todo!();
}
