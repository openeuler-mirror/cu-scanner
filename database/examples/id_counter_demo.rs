//! 持久化ID计数器演示程序

use database::{DatabaseConfig, DatabaseManager, PersistentIdCounter};
use std::sync::Arc;
use tokio::sync::Mutex;
use utils::config::AppConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    todo!()
}
