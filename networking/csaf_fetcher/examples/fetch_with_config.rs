//! 使用配置文件批量获取 CSAF 文件示例
//!
//! 演示如何从 cu-scanner.toml 配置文件读取 CSAF URL 并批量下载

use csaf_fetcher::{AsyncCsafFetcher, FetcherConfig};
use utils::config::AppConfig;

#[tokio::main]
async fn main() {
    // 初始化日志
    env_logger::init();

    println!("=== 使用配置文件批量获取 CSAF 文件示例 ===\n");

    // 1. 加载配置文件
    todo!();
}
