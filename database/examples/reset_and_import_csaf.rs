//! 清空数据库并重新导入CSAF文件的示例程序

use csaf::CSAF;
use database::{DatabaseConfig, DatabaseManager};
use parser::{csaf_to_oval, process_csaf_id};
use utils::config::AppConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("清空数据库并重新导入CSAF文件示例程序");

    // 从配置文件加载数据库配置
    todo!();
}
