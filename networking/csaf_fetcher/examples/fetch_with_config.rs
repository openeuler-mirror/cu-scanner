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
    todo!();
}
