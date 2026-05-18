//! 从 index.txt 文件批量获取 CSAF 文件示例
//!
//! 演示如何从 index.txt 文件中解析 CSAF 文件列表并批量下载

use csaf_fetcher::{CsafFetcher, FetcherConfig};

fn main() {
    // 初始化日志
    env_logger::init();

    println!("=== 从 index.txt 文件批量获取 CSAF 文件示例 ===\n");

    // 1. 创建获取器
    println!("【1. 创建获取器】");
    let config = FetcherConfig {
        timeout_secs: 60,
        max_retries: 3,
        retry_delay_ms: 1000,
        user_agent: "CSAF-Index-Fetcher/1.0".to_string(),
    };

    let fetcher = CsafFetcher::new(config).expect("创建获取器失败");
    println!("  ✓ 成功创建同步获取器");
    println!();

    // 2. 配置 URL
    todo!();
}
