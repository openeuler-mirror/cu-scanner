//! 基于数据库的ID生成器演示程序

use database::{DatabaseConfig, DatabaseIdGenerator, DatabaseManager};
use std::sync::Arc;
use tokio::sync::Mutex;
use utils::config::AppConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    todo!()
}
