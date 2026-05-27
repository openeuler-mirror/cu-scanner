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
    println!("  ✓ 成功创建自定义配置的异步获取器");
    println!("    - 超时: 60秒");
    println!("    - 最大重试: 5次");
    println!("    - 重试延迟: 2000毫秒");
    println!();

    // 3. 异步获取 CSAF 文件
    println!("【3. 异步获取 CSAF 文件】");

    let test_url = "https://www.openeuler.org/csaf/openEuler-SA-2025-1004.json";

    println!("  尝试异步获取: {}", test_url);
    println!("  注意：这是示例 URL，实际使用时需要替换为有效的 CSAF 文件 URL");

    match fetcher.fetch(test_url).await {
        Ok(csaf) => {
            println!("  ✓ 成功异步获取 CSAF 文件");
            println!("    - 文档 ID: {}", csaf.document.tracking.id);
            println!("    - 标题: {}", csaf.document.title);
            println!("    - 漏洞数量: {}", csaf.vulnerabilities.len());

            if let Some(first_vuln) = csaf.vulnerabilities.first() {
                println!("    - 第一个漏洞: {}", first_vuln.cve);
            }
        }
        Err(e) => {
            println!("  ✗ 获取失败: {}", e);
            println!("    这是预期的，因为示例 URL 可能不可用");
        }
    }
    println!();

    // 4. 异步获取并保存到文件
    println!("【4. 异步获取并保存到文件】");
    let output_path = "/tmp/csaf_async_example.json";
    println!("  如果成功获取，将异步保存到: {}", output_path);

    match fetcher.fetch_and_save(test_url, output_path).await {
        Ok(_csaf) => {
            println!("  ✓ 成功异步保存 CSAF 文件到: {}", output_path);
        }
        Err(e) => {
            println!("  ✗ 保存失败: {}", e);
        }
    }
    println!();

    // 5. 顺序批量异步获取
    println!("【5. 顺序批量异步获取】");
    let urls = vec![
        "https://example.com/csaf1.json".to_string(),
        "https://example.com/csaf2.json".to_string(),
    ];

    println!("  顺序批量获取 {} 个 URL", urls.len());
    let results = fetcher.fetch_batch(&urls).await;

    for (url, result) in results {
        match result {
            Ok(_csaf) => println!("  ✓ {}: 成功", url),
            Err(e) => println!("  ✗ {}: 失败 - {}", url, e),
        }
    }
    println!();

    // 6. 并发批量异步获取
    println!("【6. 并发批量异步获取】");
    let concurrent_urls = vec![
        "https://example.com/csaf3.json".to_string(),
        "https://example.com/csaf4.json".to_string(),
        "https://example.com/csaf5.json".to_string(),
    ];

    println!("  并发批量获取 {} 个 URL", concurrent_urls.len());
    let concurrent_results = custom_fetcher
        .fetch_batch_concurrent(&concurrent_urls)
        .await;

    for (url, result) in concurrent_results {
        match result {
            Ok(_csaf) => println!("  ✓ {}: 成功", url),
            Err(e) => println!("  ✗ {}: 失败 - {}", url, e),
        }
    }
    println!();

    println!("异步示例执行完成！");
}
