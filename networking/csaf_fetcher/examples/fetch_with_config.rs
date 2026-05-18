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
    todo!();
}
