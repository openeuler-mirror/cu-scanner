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
    todo!()
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
