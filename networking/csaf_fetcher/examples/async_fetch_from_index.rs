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
    println!();

    // 3. 异步获取索引文件
    println!("【3. 异步获取索引文件】");
    match fetcher.fetch_index(index_url).await {
        Ok(paths) => {
            println!("  ✓ 成功异步获取索引文件");
            println!("  解析到 {} 个 CSAF 文件路径", paths.len());

            // 显示前 5 个文件路径
            println!("\n  前 5 个文件路径:");
            for (i, path) in paths.iter().take(5).enumerate() {
                println!("    {}. {}", i + 1, path);
            }

            if paths.len() > 5 {
                println!("    ... 还有 {} 个文件", paths.len() - 5);
            }
        }
        Err(e) => {
            println!("  ✗ 异步获取索引文件失败: {}", e);
            println!("  这是预期的，因为示例 URL 不可用");
        }
    }
    println!();

    // 4. 顺序批量获取（慢速模式）
    println!("【4. 顺序批量异步获取（慢速模式）】");
    println!("  逐个下载文件，适合小规模或有速率限制的情况");

    match fetcher.fetch_from_index(index_url, base_url).await {
        Ok(results) => {
            println!("  ✓ 顺序批量获取完成");

            let success_count = results.iter().filter(|(_, r)| r.is_ok()).count();
            let fail_count = results.len() - success_count;

            println!("  总计: {} 个文件", results.len());
            println!("  成功: {} 个", success_count);
            println!("  失败: {} 个", fail_count);
        }
        Err(e) => {
            println!("  ✗ 顺序批量获取失败: {}", e);
        }
    }
    println!();

    // 5. 并发批量获取（快速模式）
    println!("【5. 并发批量异步获取（快速模式）】");
    println!("  并发下载所有文件，速度更快，适合大规模下载");

    match fetcher
        .fetch_from_index_concurrent(index_url, base_url)
        .await
    {
        Ok(results) => {
            println!("  ✓ 并发批量获取完成");

            let success_count = results.iter().filter(|(_, r)| r.is_ok()).count();
            let fail_count = results.len() - success_count;

            println!("  总计: {} 个文件", results.len());
            println!("  成功: {} 个", success_count);
            println!("  失败: {} 个", fail_count);

            // 显示前 3 个成功的结果
            println!("\n  前 3 个成功获取的文件:");
            for (i, (path, result)) in results
                .iter()
                .filter(|(_, r)| r.is_ok())
                .take(3)
                .enumerate()
            {
                if let Ok(csaf) = result {
                    println!("    {}. {}", i + 1, path);
                    println!("       - ID: {}", csaf.document.tracking.id);
                    println!("       - 标题: {}", csaf.document.title);
                    println!("       - 漏洞数: {}", csaf.vulnerabilities.len());
                }
            }
        }
        Err(e) => {
            println!("  ✗ 并发批量获取失败: {}", e);
            println!("  这是预期的，因为示例 URL 不可用");
        }
    }
    println!();

    // 6. 并发批量获取并保存到文件
    println!("【6. 异步并发获取并保存到文件】");
    let output_dir = "/tmp/csaf_files_async";
    println!("  输出目录: {}", output_dir);

    match fetcher
        .fetch_from_index_and_save(index_url, base_url, output_dir)
        .await
    {
        Ok(results) => {
            println!("  ✓ 异步批量获取并保存完成");

            let success_count = results.iter().filter(|(_, r)| r.is_ok()).count();
            let fail_count = results.len() - success_count;

            println!("  总计: {} 个文件", results.len());
            println!("  成功保存: {} 个", success_count);
            println!("  失败: {} 个", fail_count);

            println!("\n  保存的文件可在以下目录查看:");
            println!("    {}", output_dir);
        }
        Err(e) => {
            println!("  ✗ 异步批量保存失败: {}", e);
            println!("  这是预期的，因为示例 URL 不可用");
        }
    }
    println!();

    // 性能对比说明
    println!("【性能对比】");
    println!("  顺序模式 vs 并发模式:");
    println!("    - 顺序模式: 逐个下载，总时间 = 文件数 × 单个下载时间");
    println!("    - 并发模式: 同时下载，总时间 ≈ 最慢的单个下载时间");
    println!();
    println!("  例如下载 100 个文件:");
    println!("    - 每个文件需要 1 秒");
    println!("    - 顺序模式: 约 100 秒");
    println!("    - 并发模式: 约 1-2 秒（取决于网络和服务器）");
    println!();
    println!("  建议:");
    println!("    - 小规模（< 10 个文件）: 使用顺序模式");
    println!("    - 大规模（> 10 个文件）: 使用并发模式");
    println!("    - 有速率限制: 使用顺序模式或限制并发数");
    println!();

    println!("异步示例执行完成！");
}
