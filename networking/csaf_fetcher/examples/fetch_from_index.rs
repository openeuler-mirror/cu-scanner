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
    // 注意：这些是示例 URL，实际使用时需要替换为真实的地址
    let index_url = "http://csaf-website/index.txt";
    let base_url = "http://csaf-website";

    println!("【2. 配置 URL】");
    println!("  索引文件: {}", index_url);
    println!("  基础 URL: {}", base_url);
    println!();

    // 3. 获取索引文件
    println!("【3. 获取索引文件】");
    match fetcher.fetch_index(index_url) {
        Ok(paths) => {
            println!("  ✓ 成功获取索引文件");
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
            println!("  ✗ 获取索引文件失败: {}", e);
            println!("  这是预期的，因为示例 URL 不可用");
            println!("\n  实际使用时的示例:");
            println!("  假设 index.txt 内容如下:");
            println!("    2021/csaf-openeuler-sa-2021-1001.json");
            println!("    2021/csaf-openeuler-sa-2021-1002.json");
            println!("    2021/csaf-openeuler-sa-2021-1003.json");
            println!("\n  解析后会得到这些路径，然后拼接为完整 URL:");
            println!("    http://csaf-website/2021/csaf-openeuler-sa-2021-1001.json");
            println!("    http://csaf-website/2021/csaf-openeuler-sa-2021-1002.json");
            println!("    http://csaf-website/2021/csaf-openeuler-sa-2021-1003.json");
        }
    }
    println!();

    // 4. 批量获取 CSAF 文件
    println!("【4. 批量获取 CSAF 文件】");
    println!("  尝试从索引文件批量获取所有 CSAF 文件...");

    match fetcher.fetch_from_index(index_url, base_url) {
        Ok(results) => {
            println!("  ✓ 批量获取完成");

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
                    println!("       - 漏洞数: {}", csaf.vulnerabilities.len());
                }
            }
        }
        Err(e) => {
            println!("  ✗ 批量获取失败: {}", e);
            println!("  这是预期的，因为示例 URL 不可用");
        }
    }
    todo!();
}
