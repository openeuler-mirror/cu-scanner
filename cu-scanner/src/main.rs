//! cu-scanner主程序
//!
//! 该程序是cu-scanner工具的入口点。

use actix_server::{ServerConfig, create_default_server};
use clap::Parser;
use csaf::CSAF;
use csaf_fetcher::{AsyncCsafFetcher, FetcherConfig};
use database::{DatabaseConfig, DatabaseManager, converter};
use parser::csaf_to_oval;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;
use utils::config::AppConfig;
use utils::log::{self, LogTarget};

/// cu-scanner - 安全漏洞扫描与分析工具
#[derive(Parser, Debug)]
#[clap(
    name = "cu-scanner",
    version = "1.0",
    about = "安全漏洞扫描与分析工具"
)]
struct CliArgs {
    /// 配置文件路径
    #[clap(
        short = 'c',
        long = "config",
        default_value = "/etc/cu-scanner/cu-scanner.toml"
    )]
    config: String,

    /// CSAF文件路径（单个文件）
    #[clap(short = 'f', long = "csaf-file")]
    csaf_file: Option<String>,

    /// CSAF文件目录路径（处理目录中的所有CSAF文件）
    #[clap(short = 'D', long = "csaf-dir")]
    csaf_dir: Option<String>,

    /// 从网络获取CSAF文件（使用配置文件中的csaf_url）
    #[clap(short = 'F', long = "fetch-csaf")]
    fetch_csaf: bool,

    /// 初始化数据库（清空并重新创建所有表结构）
    #[clap(long = "init-db")]
    init_db: bool,

    /// 转换后的OVAL XML文件保存路径
    #[clap(short = 'o', long = "output", conflicts_with = "daemon")]
    output: Option<String>,

    /// 以守护进程方式运行服务
    #[clap(short = 'd', long = "daemon", conflicts_with = "output")]
    daemon: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 在配置加载之前，先初始化一个临时的stdout日志记录器
    utils::log::init_temporary_stdout_logger();
    log::info!("cu-scanner初始化中...");

    // 解析命令行参数
    let args = CliArgs::parse();

    log::info!("配置文件路径: {}", args.config);

    // 先加载配置文件以获取日志配置
    let config = match AppConfig::from_file(&args.config) {
        Ok(config) => {
            log::info!("配置文件加载成功: {}", args.config);
            config
        }
        Err(e) => {
            log::error!("配置文件加载失败: {}，使用默认配置", e);
            AppConfig::default()
        }
    };

    // 根据配置文件中的日志配置确定日志输出目标
    let log_target = if config.logging.stdout {
        LogTarget::Stdout
    } else if !config.logging.file.is_empty() {
        LogTarget::File(config.logging.file.clone())
    } else {
        LogTarget::Stdout
    };

    // 设置日志级别
    let log_level = match config.logging.level.as_str() {
        "debug" => log::Level::Debug,
        "info" => log::Level::Info,
        "warn" => log::Level::Warn,
        "error" => log::Level::Error,
        _ => log::Level::Info, // 默认为info级别
    };

    log::info!("日志系统初始化完成，输出目标: {:?}", log_target);

    // 重新初始化日志系统
    match log::init_logger_with_level_and_target(log_level, log_target) {
        Ok(_) => log::info!("日志系统重新初始化成功"),
        Err(e) => log::error!("日志系统重新初始化失败: {}", e),
    }

    // 如果指定了初始化数据库参数
    todo!();
}

/// 从CSAF文件名中提取OVAL ID
///
/// 将文件名中的最后数字-数字部分转换为数字数字形式，然后添加OVAL定义前缀
///
/// # 参数
///
/// * `filename` - CSAF文件名，例如 "csaf-openeuler-sa-2025-1004.json"
///
/// # 返回值
///
/// 返回OVAL ID，例如 "oval:org.openeuler.cu-scanner:def:20251004"
fn extract_oval_id_from_filename(filename: &str) -> Option<String> {
    todo!()
}

/// 从网络获取CSAF文件并存储到数据库
///
/// # 参数
///
/// * `config` - 应用配置
///
/// # 返回值
///
/// 返回Result<()>，成功时为()，失败时包含错误信息
pub async fn fetch_csaf_from_network(config: &AppConfig) -> Result<(), Box<dyn std::error::Error>> {
    todo!()
}

/// CSAF定时获取守护线程
///
/// 在daemon模式下按照配置的间隔时间定期从网络获取CSAF文件
///
/// # 参数
///
/// * `config` - 应用配置
async fn csaf_fetch_daemon(config: AppConfig) {
    todo!()
}

/// 初始化数据库
async fn init_database(config: &AppConfig) -> Result<(), Box<dyn std::error::Error>> {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_oval_id_from_filename() {
        todo!()
    }
}
