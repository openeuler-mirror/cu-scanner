//! CSAF 同步获取示例
//!
//! 演示如何使用同步方式获取 CSAF 文件

use csaf_fetcher::{CsafFetcher, FetcherConfig};

fn main() {
    // 初始化日志
    env_logger::init();

    println!("=== CSAF 同步获取示例 ===\n");

    // 1. 使用默认配置创建获取器
    println!("【1. 创建获取器】");
    let fetcher = CsafFetcher::with_defaults().expect("创建获取器失败");
    println!("  ✓ 成功创建同步获取器（默认配置）");
    println!();

    // 2. 使用自定义配置
    println!("【2. 自定义配置】");
    let custom_config = FetcherConfig {
        timeout_secs: 60,
        max_retries: 5,
        retry_delay_ms: 2000,
        user_agent: "My-CSAF-Client/1.0".to_string(),
    };

    let custom_fetcher = CsafFetcher::new(custom_config).expect("创建自定义获取器失败");
    println!("  ✓ 成功创建自定义配置的获取器");
    println!("    - 超时: 60秒");
    println!("    - 最大重试: 5次");
    println!("    - 重试延迟: 2000毫秒");
    println!();

    // 3. 获取 CSAF 文件（使用本地测试文件的 URL 或实际 URL）
    println!("【3. 获取 CSAF 文件（使用自定义配置的获取器）】");

    // 注意：这里需要替换为实际的 CSAF URL
    todo!();
}
