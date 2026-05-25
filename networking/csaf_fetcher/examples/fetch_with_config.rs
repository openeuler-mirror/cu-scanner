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
    todo!();
}
