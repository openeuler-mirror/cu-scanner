//! 测试从数据库导出OVAL XML时是否包含OS检查信息

use database::{DatabaseConfig, DatabaseManager};
use utils::config::AppConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("测试从数据库导出OVAL XML时是否包含OS检查信息\n");

    // 从配置文件加载数据库配置
    todo!();
}
