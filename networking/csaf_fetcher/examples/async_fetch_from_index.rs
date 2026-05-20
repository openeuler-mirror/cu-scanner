//! 从 index.txt 文件并发批量获取 CSAF 文件示例（异步版本）
//!
//! 演示如何使用异步方式高效并发获取大量 CSAF 文件

use csaf_fetcher::{AsyncCsafFetcher, FetcherConfig};

#[tokio::main]
async fn main() {
    // 初始化日志
    env_logger::init();

    println!("=== 从 index.txt 异步并发批量获取 CSAF 文件示例 ===\n");

    // 1. 创建异步获取器
    println!("【1. 创建异步获取器】");
    let config = FetcherConfig {
        timeout_secs: 60,
        max_retries: 3,
        retry_delay_ms: 1000,
        user_agent: "CSAF-Async-Fetcher/1.0".to_string(),
    };

    let fetcher = AsyncCsafFetcher::new(config).expect("创建异步获取器失败");
    println!("  ✓ 成功创建异步获取器");
    println!();

    // 2. 配置 URL
    let index_url = "http://csaf-website/index.txt";
    let base_url = "http://csaf-website";

    println!("【2. 配置 URL】");
    println!("  索引文件: {}", index_url);
    println!("  基础 URL: {}", base_url);
    todo!();
}
