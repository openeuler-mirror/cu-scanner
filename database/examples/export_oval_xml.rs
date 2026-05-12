//! 从数据库导出OVAL定义为XML文件的示例程序

use database::{DatabaseConfig, DatabaseManager};
use std::env;
use std::fs::File;
use std::io::Write;
use utils::config::AppConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    todo!()
}
