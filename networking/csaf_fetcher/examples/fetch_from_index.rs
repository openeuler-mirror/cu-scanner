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
    println!();

    // 5. 批量获取并保存到文件
    println!("【5. 批量获取并保存到文件】");
    let output_dir = "/tmp/csaf_files";
    println!("  输出目录: {}", output_dir);

    match fetcher.fetch_from_index_and_save(index_url, base_url, output_dir) {
        Ok(results) => {
            println!("  ✓ 批量获取并保存完成");

            let success_count = results.iter().filter(|(_, r)| r.is_ok()).count();
            let fail_count = results.len() - success_count;

            println!("  总计: {} 个文件", results.len());
            println!("  成功保存: {} 个", success_count);
            println!("  失败: {} 个", fail_count);

            // 显示保存的文件名示例
            println!("\n  文件保存规则:");
            println!("    原路径: 2021/csaf-openeuler-sa-2021-1001.json");
            println!("    保存为: /tmp/csaf_files/2021_csaf-openeuler-sa-2021-1001.json");
        }
        Err(e) => {
            println!("  ✗ 批量保存失败: {}", e);
            println!("  这是预期的，因为示例 URL 不可用");
        }
    }
    println!();

    // 使用说明
    println!("【使用说明】");
    println!("  实际使用时，请替换以下变量:");
    println!("    - index_url: index.txt 文件的实际 URL");
    println!("    - base_url: CSAF 文件的基础 URL");
    println!("    - output_dir: 文件保存目录");
    println!();
    println!("  index.txt 文件格式:");
    println!("    每行一个相对路径，例如:");
    println!("      2021/csaf-openeuler-sa-2021-1001.json");
    println!("      2021/csaf-openeuler-sa-2021-1002.json");
    println!("      2022/csaf-openeuler-sa-2022-1001.json");
    println!();
    println!("  程序会自动:");
    println!("    1. 下载并解析 index.txt 文件");
    println!("    2. 提取所有 .json 文件路径");
    println!("    3. 拼接完整 URL 并下载每个 CSAF 文件");
    println!("    4. 可选择保存到本地文件系统");
    println!();

    println!("示例执行完成！");
}
