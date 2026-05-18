//! CSAF 异步获取示例
//!
//! 演示如何使用异步方式获取 CSAF 文件

use csaf_fetcher::{AsyncCsafFetcher, FetcherConfig};

#[tokio::main]
async fn main() {
    // 初始化日志
    env_logger::init();

    println!("=== CSAF 异步获取示例 ===\n");

    // 1. 使用默认配置创建异步获取器
    println!("【1. 创建异步获取器】");
    let fetcher = AsyncCsafFetcher::with_defaults().expect("创建异步获取器失败");
    println!("  ✓ 成功创建异步获取器（默认配置）");
    println!();
    todo!();
}
