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
    if args.init_db {
        return init_database(&config).await;
    }

    if args.daemon {
        log::info!("以守护进程模式运行服务...");

        // 如果配置了csaf_url，启动CSAF定时获取线程
        if config.csaf_url.is_some() {
            log::info!("检测到csaf_url配置，启动CSAF定时获取线程");
            let config_clone = config.clone();
            tokio::spawn(async move {
                csaf_fetch_daemon(config_clone).await;
            });
        } else {
            log::info!("未配置csaf_url，跳过CSAF定时获取");
        }

        // 创建服务器配置
        let port = config.server.port.parse::<u16>().unwrap_or(8091);
        log::info!("服务器将监听: {}:{}", config.server.address, port);

        let server_config = ServerConfig {
            database_config: DatabaseConfig::new(
                &config.database.host,
                config.database.port,
                &config.database.database,
                &config.database.username,
                &config.database.password,
            ),
            address: config.server.address.clone(),
            port,
            api_group_name: config.api.group_name.clone(),
        };

        // 启动网络服务（这会阻塞当前任务）
        create_default_server(server_config).await?;
        return Ok(());
    }

    log::info!("cu-scanner主程序启动");

    // 如果指定了从网络获取CSAF文件
    if args.fetch_csaf {
        fetch_csaf_from_network(&config).await?;
    }
    // 如果提供了CSAF目录路径，则处理目录中的所有CSAF文件
    else if let Some(csaf_dir_path) = args.csaf_dir {
        log::info!("处理CSAF目录: {}", csaf_dir_path);

        // 创建数据库连接
        let db_config = DatabaseConfig::new(
            &config.database.host,
            config.database.port,
            &config.database.database,
            &config.database.username,
            &config.database.password,
        );

        let db_manager = match DatabaseManager::new(&db_config).await {
            Ok(manager) => Arc::new(Mutex::new(manager)),
            Err(e) => {
                log::error!("数据库连接失败: {}", e);
                return Ok(());
            }
        };

        // 读取目录中的所有CSAF文件
        let dir_entries = match fs::read_dir(&csaf_dir_path) {
            Ok(entries) => entries,
            Err(e) => {
                log::error!("读取目录失败: {}", e);
                return Ok(());
            }
        };

        for entry in dir_entries {
            if let Ok(entry) = entry {
                let path = entry.path();

                // 只处理.json文件
                if path.extension().and_then(|s| s.to_str()) == Some("json") {
                    let file_name = path.file_name().unwrap().to_str().unwrap();
                    log::info!("发现CSAF文件: {}", file_name);

                    // 从文件名提取OVAL ID
                    if let Some(oval_id) = extract_oval_id_from_filename(file_name) {
                        log::info!("从文件名提取的OVAL ID: {}", oval_id);

                        // 查询数据库中是否已存在该ID
                        let exists = {
                            let db = db_manager.lock().await;
                            match db.get_oval_definition(&oval_id).await {
                                Ok(Some(_)) => true,
                                Ok(None) => false,
                                Err(e) => {
                                    log::error!("查询数据库失败: {}", e);
                                    continue;
                                }
                            }
                        };

                        if exists {
                            log::info!("OVAL定义 {} 已存在于数据库中，跳过", oval_id);
                            continue;
                        }

                        log::info!("OVAL定义 {} 不存在，开始处理文件", oval_id);

                        // 读取并转换CSAF文件
                        match CSAF::from_file(path.to_str().unwrap()) {
                            Ok(csaf) => {
                                log::info!("CSAF文件加载成功: {}", csaf.document.title);

                                // 转换CSAF到OVAL（使用数据库计数器）
                                match database::csaf_to_oval_with_default_db_counter(
                                    &csaf,
                                    db_manager.clone(),
                                )
                                .await
                                {
                                    Ok(oval) => {
                                        log::info!("CSAF到OVAL转换成功");

                                        // 保存到数据库
                                        if let Some(definition) = oval.definitions.items.first() {
                                            let (
                                                db_definition,
                                                references,
                                                cves,
                                                tests,
                                                objects,
                                                states,
                                            ) = converter::convert_full_oval_definition(
                                                definition,
                                                &oval.tests,
                                                &oval.objects,
                                                &oval.states,
                                            );

                                            let mut db = db_manager.lock().await;
                                            match db
                                                .save_full_oval_definition(
                                                    &db_definition,
                                                    &references,
                                                    &cves,
                                                    &tests,
                                                    &objects,
                                                    &states,
                                                )
                                                .await
                                            {
                                                Ok(_) => {
                                                    log::info!(
                                                        "OVAL定义保存成功: {}",
                                                        db_definition.id
                                                    );
                                                }
                                                Err(e) => {
                                                    log::error!("保存OVAL定义到数据库失败: {}", e);
                                                }
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        log::error!("CSAF到OVAL转换失败: {}", e);
                                    }
                                }
                            }
                            Err(e) => {
                                log::error!("加载CSAF文件失败: {}", e);
                            }
                        }
                    } else {
                        log::warn!("无法从文件名提取OVAL ID: {}", file_name);
                    }
                }
            }
        }
    }
    // 如果提供了CSAF文件路径，则处理CSAF文件
    else if let Some(csaf_file_path) = args.csaf_file {
        log::info!("处理CSAF文件: {}", csaf_file_path);

        // 读取CSAF文件
        match CSAF::from_file(&csaf_file_path) {
            Ok(csaf) => {
                log::info!("CSAF文件加载成功: {}", csaf.document.title);

                // 转换CSAF到OVAL
                match csaf_to_oval(&csaf) {
                    Ok(oval) => {
                        log::info!("CSAF到OVAL转换成功");

                        // 如果提供了输出路径，则保存转换结果
                        if let Some(output_path) = args.output {
                            log::info!("转换结果将保存到: {}", output_path);

                            // 将OVAL转换为XML字符串
                            match oval.to_oval_string() {
                                Ok(xml_content) => {
                                    // 保存到文件
                                    match fs::write(&output_path, xml_content) {
                                        Ok(_) => {
                                            log::info!("OVAL XML文件保存成功: {}", output_path);
                                        }
                                        Err(e) => {
                                            log::error!("保存OVAL XML文件失败: {}", e);
                                            return Ok(());
                                        }
                                    }
                                }
                                Err(e) => {
                                    log::error!("将OVAL转换为XML字符串失败: {}", e);
                                    return Ok(());
                                }
                            }
                        } else {
                            // 如果没有指定输出文件，则输出到标准输出
                            match oval.to_oval_string() {
                                Ok(xml_content) => {
                                    println!("{}", xml_content);
                                    log::info!("OVAL XML内容已输出到标准输出");
                                }
                                Err(e) => {
                                    log::error!("将OVAL转换为XML字符串失败: {}", e);
                                    return Ok(());
                                }
                            }
                        }
                    }
                    Err(e) => {
                        log::error!("CSAF到OVAL转换失败: {}", e);
                        return Ok(());
                    }
                }
            }
            Err(e) => {
                log::error!("加载CSAF文件失败: {}", e);
                return Ok(());
            }
        }
    } else {
        log::info!("未提供CSAF文件路径，跳过CSAF处理");
    }

    // 程序执行完成
    log::info!("cu-scanner执行完成");
    Ok(())
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
    // 移除文件扩展名
    let name_without_ext = Path::new(filename).file_stem()?.to_str()?;

    // 按减号分割
    let parts: Vec<&str> = name_without_ext.split('-').collect();

    // 需要至少有2个数字部分
    if parts.len() < 2 {
        return None;
    }

    // 从右向左查找最后两个数字部分
    let last_part = parts[parts.len() - 1];
    let second_last_part = parts[parts.len() - 2];

    // 检查最后两个部分是否都是数字
    if last_part.chars().all(|c| c.is_ascii_digit())
        && second_last_part.chars().all(|c| c.is_ascii_digit())
    {
        // 合并最后两个数字部分，去掉减号
        let numeric_id = format!("{}{}", second_last_part, last_part);

        // 添加OVAL定义前缀
        Some(format!("{}{}", oval::CU_LINUX_SA_DEF_PREFIX, numeric_id))
    } else {
        None
    }
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
    log::info!("从网络获取CSAF文件");

    // 检查配置文件中是否有csaf_url配置
    if config.csaf_url.is_none() {
        log::error!("配置文件中未找到csaf_url配置，无法从网络获取CSAF文件");
        return Ok(());
    }

    let csaf_url = config.csaf_url.as_ref().unwrap();
    log::info!("CSAF索引URL: {}", csaf_url.url);

    // 从URL中提取基础URL（去掉index.txt部分）
    let base_url = if let Some(pos) = csaf_url.url.rfind('/') {
        &csaf_url.url[..pos]
    } else {
        log::error!("无效的CSAF URL格式: {}", csaf_url.url);
        return Ok(());
    };
    log::info!("基础URL: {}", base_url);

    // 创建数据库连接
    let db_config = DatabaseConfig::new(
        &config.database.host,
        config.database.port,
        &config.database.database,
        &config.database.username,
        &config.database.password,
    );

    let db_manager = match DatabaseManager::new(&db_config).await {
        Ok(manager) => Arc::new(Mutex::new(manager)),
        Err(e) => {
            log::error!("数据库连接失败: {}", e);
            return Ok(());
        }
    };

    // 创建异步CSAF获取器
    let fetcher_config = FetcherConfig::default();
    let fetcher = match AsyncCsafFetcher::new(fetcher_config) {
        Ok(f) => f,
        Err(e) => {
            log::error!("创建CSAF获取器失败: {}", e);
            return Ok(());
        }
    };

    // 创建数据库检查回调函数
    let db_manager_clone = db_manager.clone();
    let check_exists: csaf_fetcher::AsyncCheckCallback = Box::new(move |path: String| {
        let db_manager = db_manager_clone.clone();
        Box::pin(async move {
            // 从路径中提取文件名并生成OVAL ID
            let filename = path.replace('/', "_");
            if let Some(oval_id) = extract_oval_id_from_filename(&filename) {
                let db = db_manager.lock().await;
                match db.get_oval_definition(&oval_id).await {
                    Ok(Some(_)) => {
                        log::debug!("OVAL定义 {} 已存在于数据库中", oval_id);
                        true
                    }
                    Ok(None) => {
                        log::debug!("OVAL定义 {} 不存在于数据库中", oval_id);
                        false
                    }
                    Err(e) => {
                        log::error!("查询数据库失败: {}", e);
                        false // 出错时仍然下载
                    }
                }
            } else {
                log::warn!("无法从文件名提取OVAL ID: {}", filename);
                false // 无法提取ID时仍然下载
            }
        })
    });

    // 使用带数据库检查的方法批量获取CSAF文件
    let results = match fetcher.fetch_from_index_with_check(&csaf_url.url, base_url, check_exists).await {
        Ok(r) => r,
        Err(e) => {
            log::error!("从网络获取CSAF文件失败: {}", e);
            return Ok(());
        }
    };

    log::info!("共获取到 {} 个新的CSAF文件", results.len());

    // 处理获取到的CSAF文件
    for (path, csaf_result) in results {
        match csaf_result {
            Ok(csaf) => {
                log::info!("CSAF文件获取成功: {} - {}", path, csaf.document.title);

                // 转换CSAF到OVAL（使用数据库计数器）
                match database::csaf_to_oval_with_default_db_counter(&csaf, db_manager.clone()).await {
                    Ok(oval) => {
                        log::info!("CSAF到OVAL转换成功");

                        // 保存到数据库
                        if let Some(definition) = oval.definitions.items.first() {
                            let (db_definition, references, cves, tests, objects, states) =
                                converter::convert_full_oval_definition(
                                    definition,
                                    &oval.tests,
                                    &oval.objects,
                                    &oval.states,
                                );

                            let mut db = db_manager.lock().await;
                            match db
                                .save_full_oval_definition(
                                    &db_definition,
                                    &references,
                                    &cves,
                                    &tests,
                                    &objects,
                                    &states,
                                )
                                .await
                            {
                                Ok(_) => {
                                    log::info!("OVAL定义保存成功: {}", db_definition.id);
                                }
                                Err(e) => {
                                    log::error!("保存OVAL定义到数据库失败: {}", e);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        log::error!("CSAF到OVAL转换失败: {}", e);
                    }
                }
            }
            Err(e) => {
                log::error!("获取CSAF文件失败 {}: {}", path, e);
            }
        }
    }

    log::info!("网络获取CSAF文件完成");
    Ok(())
}

/// CSAF定时获取守护线程
///
/// 在daemon模式下按照配置的间隔时间定期从网络获取CSAF文件
///
/// # 参数
///
/// * `config` - 应用配置
async fn csaf_fetch_daemon(config: AppConfig) {
    log::info!("CSAF定时获取线程启动");

    // 获取定时间隔配置
    let fetch_interval = config
        .csaf_url
        .as_ref()
        .map(|c| c.fetch_interval_secs)
        .unwrap_or(3600);

    log::info!("CSAF定时获取间隔: {} 秒", fetch_interval);

    loop {
        log::info!("开始执行CSAF定时获取任务");

        // 执行获取操作
        match fetch_csaf_from_network(&config).await {
            Ok(_) => {
                log::info!("CSAF定时获取任务执行成功");
            }
            Err(e) => {
                log::error!("CSAF定时获取任务执行失败: {}", e);
            }
        }

        // 等待下次执行
        log::info!("等待 {} 秒后执行下次获取", fetch_interval);
        tokio::time::sleep(tokio::time::Duration::from_secs(fetch_interval)).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_oval_id_from_filename() {
        // 测试标准格式的CSAF文件名
        let result = extract_oval_id_from_filename("csaf-openeuler-sa-2025-1004.json");
        assert!(result.is_some());
        assert_eq!(
            result.unwrap(),
            format!("{}20251004", oval::CU_LINUX_SA_DEF_PREFIX)
        );

        // 测试另一个格式
        let result2 = extract_oval_id_from_filename("csaf-openeuler-sa-2025-1009.json");
        assert!(result2.is_some());
        assert_eq!(
            result2.unwrap(),
            format!("{}20251009", oval::CU_LINUX_SA_DEF_PREFIX)
        );

        // 测试不符合格式的文件名（最后一个部分不是数字）
        let result3 = extract_oval_id_from_filename("csaf-openeuler-sa.json");
        assert!(result3.is_none());

        // 测试只有一个数字部分
        let result4 = extract_oval_id_from_filename("csaf-1004.json");
        assert!(result4.is_none());

        // 测试没有扩展名
        let result5 = extract_oval_id_from_filename("csaf-openeuler-sa-2025-1004");
        assert!(result5.is_some());
        assert_eq!(
            result5.unwrap(),
            format!("{}20251004", oval::CU_LINUX_SA_DEF_PREFIX)
        );
    }
}

/// 初始化数据库
async fn init_database(config: &AppConfig) -> Result<(), Box<dyn std::error::Error>> {
    log::info!("使用配置初始化数据库");
    log::info!(
        "数据库主机: {}:{}",
        config.database.host,
        config.database.port
    );
    log::info!("数据库名称: {}", config.database.database);

    let db_config = DatabaseConfig::new(
        &config.database.host,
        config.database.port,
        &config.database.database,
        &config.database.username,
        &config.database.password,
    );

    log::info!("正在连接数据库...");
    let mut db_manager = match DatabaseManager::new(&db_config).await {
        Ok(manager) => {
            log::info!("数据库连接成功");
            manager
        }
        Err(e) => {
            log::error!("数据库连接失败: {}", e);
            return Err(e.into());
        }
    };

    log::info!("正在清空并重新创建数据库表结构...");
    match db_manager.reinit_tables().await {
        Ok(_) => {
            log::info!("数据库表结构初始化成功");
            log::info!("已创建以下表:");
            log::info!("  - os_info (操作系统信息表)");
            log::info!("  - oval_definitions (OVAL定义表)");
            log::info!("  - references_info (引用信息表)");
            log::info!("  - cves (CVE信息表)");
            log::info!("  - rpminfo_tests (RPM测试表)");
            log::info!("  - rpminfo_objects (RPM对象表)");
            log::info!("  - rpminfo_states (RPM状态表，包含EVR信息)");
            log::info!("  - id_counters (ID计数器表)");

            // 初始化OS信息数据
            log::info!("正在初始化操作系统信息数据...");
            match db_manager.init_os_info_data().await {
                Ok(_) => {
                    log::info!("操作系统信息数据初始化成功");
                    log::info!("已初始化以下操作系统:");
                    log::info!("  - openEuler 20.03 (oe1)");
                    log::info!("  - openEuler 22.03 (oe2203)");
                    log::info!("  - Red Hat Enterprise Linux 7 (el7)");
                    log::info!("  - Red Hat Enterprise Linux 8 (el8)");
                }
                Err(e) => {
                    log::error!("操作系统信息数据初始化失败: {}", e);
                    return Err(e.into());
                }
            }

            Ok(())
        }
        Err(e) => {
            log::error!("数据库表结构初始化失败: {}", e);
            Err(e.into())
        }
    }
}
