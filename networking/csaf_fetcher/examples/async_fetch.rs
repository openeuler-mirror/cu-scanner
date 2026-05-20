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

    // 2. 使用自定义配置
    println!("【2. 自定义配置】");
    let custom_config = FetcherConfig {
        timeout_secs: 60,
        max_retries: 5,
        retry_delay_ms: 2000,
        user_agent: "My-Async-CSAF-Client/1.0".to_string(),
    };

    let custom_fetcher = AsyncCsafFetcher::new(custom_config).expect("创建自定义异步获取器失败");
    todo!();
}
