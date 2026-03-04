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
    println!("【1. 加载配置文件】");
    let config_path = "/home/fatmouse/workspace/cu-scanner/config/cu-scanner.toml";

    let app_config = match AppConfig::from_file(config_path) {
        Ok(config) => {
            println!("  ✓ 成功加载配置文件: {}", config_path);
            config
        }
        Err(e) => {
            eprintln!("  ✗ 加载配置文件失败: {}", e);
            return;
        }
    };

    // 2. 检查 csaf_url 配置
    println!("\n【2. 检查 CSAF URL 配置】");
    let csaf_url_config = match &app_config.csaf_url {
        Some(config) => {
            println!("  ✓ 找到 CSAF URL 配置");
            println!("  URL: {}", config.url);
            config
        }
        None => {
            eprintln!("  ✗ 配置文件中未找到 csaf_url 配置");
            return;
        }
    };

    // 3. 解析 index.txt URL 和基础 URL
    println!("\n【3. 解析 URL】");
    let index_url = &csaf_url_config.url;

    // 从 index.txt URL 中提取基础 URL
    // 例如：https://example.com/security/data/csaf/advisories/index.txt
    //   -> https://example.com/security/data/csaf/advisories
    let base_url = if let Some(pos) = index_url.rfind('/') {
        &index_url[..pos]
    } else {
        eprintln!("  ✗ 无法解析基础 URL");
        return;
    };

    println!("  索引文件: {}", index_url);
    println!("  基础 URL: {}", base_url);

    // 4. 创建 CSAF 获取器
    println!("\n【4. 创建 CSAF 获取器】");
    let fetcher_config = FetcherConfig {
        timeout_secs: 60,
        max_retries: 3,
        retry_delay_ms: 1000,
        user_agent: "cu-scanner/1.0".to_string(),
    };

    let fetcher = match AsyncCsafFetcher::new(fetcher_config) {
        Ok(f) => {
            println!("  ✓ 成功创建异步获取器");
            f
        }
        Err(e) => {
            eprintln!("  ✗ 创建获取器失败: {}", e);
            return;
        }
    };

    // 5. 获取 index.txt 文件
    println!("\n【5. 获取 index.txt 文件】");
    let paths = match fetcher.fetch_index(index_url).await {
        Ok(paths) => {
            println!("  ✓ 成功获取 index.txt 文件");
            println!("  解析到 {} 个 CSAF 文件路径", paths.len());

            // 显示前 5 个路径
            if !paths.is_empty() {
                println!("\n  前 5 个文件路径:");
                for (i, path) in paths.iter().take(5).enumerate() {
                    println!("    {}. {}", i + 1, path);
                }
                if paths.len() > 5 {
                    println!("    ... 还有 {} 个文件", paths.len() - 5);
                }
            }

            paths
        }
        Err(e) => {
            eprintln!("  ✗ 获取 index.txt 失败: {}", e);
            return;
        }
    };

    if paths.is_empty() {
        println!("\n  ⚠ index.txt 文件为空，没有需要下载的 CSAF 文件");
        return;
    }

    // 6. 并发批量获取 CSAF 文件（演示模式：只获取前 3 个）
    println!("\n【6. 并发批量获取 CSAF 文件（演示模式）】");

    // 为了演示，只获取前 3 个文件
    let demo_count = std::cmp::min(3, paths.len());
    println!("  演示模式：只获取前 {} 个文件", demo_count);

    match fetcher
        .fetch_from_index_concurrent(index_url, base_url)
        .await
    {
        Ok(results) => {
            let success_count = results.iter().filter(|(_, r)| r.is_ok()).count();
            let fail_count = results.len() - success_count;

            println!("  ✓ 批量获取完成");
            println!("  总计: {} 个文件", results.len());
            println!("  成功: {} 个", success_count);
            println!("  失败: {} 个", fail_count);

            // 显示成功获取的文件信息
            if success_count > 0 {
                println!("\n  成功获取的文件:");
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

            // 显示失败的文件
            if fail_count > 0 {
                println!("\n  失败的文件:");
                for (i, (path, result)) in results
                    .iter()
                    .filter(|(_, r)| r.is_err())
                    .take(3)
                    .enumerate()
                {
                    if let Err(e) = result {
                        println!("    {}. {}", i + 1, path);
                        println!("       错误: {}", e);
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("  ✗ 批量获取失败: {}", e);
        }
    }

    // 7. 保存到本地（可选）
    println!("\n【7. 保存到本地（可选）】");
    let output_dir = "/tmp/csaf_files";
    println!("  如果需要保存文件，可以使用 fetch_from_index_and_save 方法");
    println!("  输出目录: {}", output_dir);
    println!("  示例代码:");
    println!(
        "    fetcher.fetch_from_index_and_save(index_url, base_url, \"{}\").await?;",
        output_dir
    );

    println!("\n示例执行完成！");
    println!("\n提示:");
    println!("  - 完整获取所有文件，请移除演示模式限制");
    println!("  - 使用 RUST_LOG=debug 环境变量查看详细日志");
    println!("  - 配置文件路径: {}", config_path);
}
