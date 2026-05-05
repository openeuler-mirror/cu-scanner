//! 日志模块
//!
//! 提供统一的日志记录接口，支持输出到标准输出或文件

use chrono::Local;
use log::{LevelFilter, Metadata, Record, SetLoggerError};
use std::env;
use std::fs::OpenOptions;
use std::io::Write;
use std::sync::RwLock;

// 全局日志目标配置
static GLOBAL_LOG_TARGET: RwLock<Option<LogTarget>> = RwLock::new(None);

// 重新导出Level类型和常用的日志宏，以便其他模块可以使用
pub use log::{Level, debug, error, info, trace, warn};

/// 日志输出目标
#[derive(Debug, Clone)]
pub enum LogTarget {
    /// 标准输出
    Stdout,
    /// 文件输出
    File(String),
}

/// CU Scanner日志记录器结构体
pub struct CUScannerLogger {
    target: LogTarget,
}

impl Default for CUScannerLogger {
    fn default() -> Self {
        Self::new()
    }
}

impl CUScannerLogger {
    /// 创建新的日志记录器实例
    pub fn new() -> Self {
        let target = if let Ok(log_file) = env::var("CU_SCANNER_LOG_FILE") {
            LogTarget::File(log_file)
        } else {
            LogTarget::Stdout
        };

        CUScannerLogger { target }
    }

    /// 创建新的日志记录器实例，指定输出目标
    pub fn with_target(target: LogTarget) -> Self {
        CUScannerLogger { target }
    }

    /// 初始化日志记录器
    pub fn init(level: Level) -> Result<(), SetLoggerError> {
        let logger = Self::new();
        log::set_logger(Box::leak(Box::new(logger)))
            .map(|()| log::set_max_level(level.to_level_filter()))
    }

    /// 初始化日志记录器，指定输出目标
    pub fn init_with_target(level: Level, target: LogTarget) -> Result<(), SetLoggerError> {
        // 更新全局日志目标配置
        if let Ok(mut guard) = GLOBAL_LOG_TARGET.write() {
            *guard = Some(target.clone());
        }

        let logger = Self::with_target(target);
        log::set_logger(Box::leak(Box::new(logger)))
            .map(|()| log::set_max_level(level.to_level_filter()))
    }

    /// 获取日志输出目标
    pub fn target(&self) -> &LogTarget {
        &self.target
    }
}

impl log::Log for CUScannerLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            // 使用指定的格式：[2025-10-24 11:24:00] [INFO] module - message
            let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
            let log_line = format!(
                "[{}] [{}] {} - {}\n",
                timestamp,
                record.level(),
                record.target(),
                record.args()
            );

            // 获取当前的日志目标配置
            let target = if let Ok(guard) = GLOBAL_LOG_TARGET.read() {
                guard.clone()
            } else {
                None
            };

            // 如果全局目标未设置，使用当前记录器的目标
            let effective_target = target.unwrap_or_else(|| self.target.clone());

            match effective_target {
                LogTarget::File(path) => {
                    // 写入文件
                    let file = OpenOptions::new().create(true).append(true).open(&path);
                    match file {
                        Ok(mut f) => {
                            let _ = f.write_all(log_line.as_bytes());
                            let _ = f.flush();
                        }
                        Err(e) => {
                            eprintln!("Failed to open log file {}: {}, falling back to stdout", path, e);
                            print!("{}", log_line);
                        }
                    }
                }
                LogTarget::Stdout => {
                    // 写入标准输出
                    print!("{}", log_line);
                }
            }
        }
    }

    fn flush(&self) {
        // 由于我们每次写入都flush，这里不需要额外操作
    }
}

/// 初始化日志系统
pub fn init_logger() {
    // 忽略重复初始化的错误
    let _ = CUScannerLogger::init(Level::Info);
}

/// 初始化日志系统并设置日志级别
pub fn init_logger_with_level(level: Level) {
    // 忽略重复初始化的错误
    let _ = CUScannerLogger::init(level);
}

/// 初始化日志系统，指定输出目标
pub fn init_logger_with_target(target: LogTarget) {
    todo!()
}

/// 初始化日志系统，指定日志级别和输出目标
pub fn init_logger_with_level_and_target(level: Level, target: LogTarget) -> Result<(), SetLoggerError> {
    todo!()
}

/// 创建一个临时的stdout日志记录器，用于在配置加载之前记录日志
pub fn init_temporary_stdout_logger() {
    todo!()
}
