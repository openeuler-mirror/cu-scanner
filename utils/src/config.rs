//! 配置文件处理模块

use log::info;
use serde::{Deserialize, Serialize};
use std::fs;

/// 数据库配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub database: String,
    pub username: String,
    pub password: String,
}

/// 日志配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub file: String,
    /// 当设置为true时，日志输出到标准输出，忽略file配置
    #[serde(default)]
    pub stdout: bool,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            file: String::new(),
            stdout: false,
        }
    }
}

/// API配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    pub group_name: String,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            group_name: "api".to_string(),
        }
    }
}

/// 包配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageConfig {
    /// epoch文件路径
    #[serde(default = "default_epoch_file")]
    pub epoch_file: String,
    /// 是否使用yum
    #[serde(default)]
    pub use_yum: bool,
    /// 是否使用额外的yum源
    #[serde(default)]
    pub use_extra_yum: bool,
}

fn default_epoch_file() -> String {
    "/etc/cu-scanner/rpm_epoch.json".to_string()
}

impl Default for PackageConfig {
    fn default() -> Self {
        Self {
            epoch_file: default_epoch_file(),
            use_yum: false,
            use_extra_yum: false,
        }
    }
}

/// 服务器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// 服务器端口
    pub port: String,
    /// 服务器地址，默认为0.0.0.0
    #[serde(default = "default_server_address")]
    pub address: String,
}

fn default_server_address() -> String {
    "0.0.0.0".to_string()
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            port: "8091".to_string(),
            address: default_server_address(),
        }
    }
}

/// CSAF URL配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CsafUrlConfig {
    /// index.txt文件的URL地址
    pub url: String,
    /// 定时获取间隔（秒），默认为3600秒（1小时）
    #[serde(default = "default_fetch_interval")]
    pub fetch_interval_secs: u64,
}

fn default_fetch_interval() -> u64 {
    3600 // 默认1小时
}

impl Default for CsafUrlConfig {
    fn default() -> Self {
        Self {
            url: String::new(),
            fetch_interval_secs: default_fetch_interval(),
        }
    }
}

/// 应用配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub database: DatabaseConfig,
    #[serde(default)]
    pub csaf_db: Option<DatabaseConfig>,
    #[serde(default)]
    pub csaf_url: Option<CsafUrlConfig>,
    #[serde(default)]
    pub logging: LoggingConfig,
    #[serde(default)]
    pub api: ApiConfig,
    #[serde(default)]
    pub server: ServerConfig,
    #[serde(default)]
    pub package: PackageConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            database: DatabaseConfig {
                host: "localhost".to_string(),
                port: 5432,
                database: "cu_scanner".to_string(),
                username: "user".to_string(),
                password: "password".to_string(),
            },
            csaf_db: None,
            csaf_url: None,
            logging: LoggingConfig::default(),
            api: ApiConfig::default(),
            server: ServerConfig::default(),
            package: PackageConfig::default(),
        }
    }
}

impl AppConfig {
    /// 从文件加载配置
    ///
    /// # 参数
    ///
    /// * `path` - 配置文件路径
    ///
    /// # 返回值
    ///
    /// 返回Result<AppConfig, Box<dyn std::error::Error>>，成功时包含配置信息，失败时包含错误信息
    pub fn from_file<P: AsRef<std::path::Path>>(
        path: P,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        info!("从文件加载配置: {:?}", path.as_ref());
        let contents = fs::read_to_string(path)?;
        let mut config: AppConfig = toml::from_str(&contents)?;

        // 如果没有显式设置stdout且file为空，则默认输出到标准输出
        if !config.logging.stdout && config.logging.file.is_empty() {
            config.logging.stdout = true;
        }

        info!("配置加载成功");
        Ok(config)
    }

    /// 保存配置到文件
    ///
    /// # 参数
    ///
    /// * `path` - 配置文件路径
    ///
    /// # 返回值
    ///
    /// 返回Result<(), Box<dyn std::error::Error>>，成功时返回空元组，失败时包含错误信息
    pub fn save_to_file<P: AsRef<std::path::Path>>(
        &self,
        path: P,
    ) -> Result<(), Box<dyn std::error::Error>> {
        info!("保存配置到文件: {:?}", path.as_ref());
        let contents = toml::to_string_pretty(self)?;
        fs::write(path, contents)?;
        info!("配置保存成功");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_serialization() {
        let config = AppConfig {
            database: DatabaseConfig {
                host: "localhost".to_string(),
                port: 5432,
                database: "test_db".to_string(),
                username: "user".to_string(),
                password: "pass".to_string(),
            },
            csaf_db: None,
            csaf_url: None,
            logging: LoggingConfig {
                level: "info".to_string(),
                file: "tmp/app.log".to_string(),
                stdout: false,
            },
            api: ApiConfig {
                group_name: "test_api".to_string(),
            },
            server: ServerConfig::default(),
            package: PackageConfig::default(),
        };

        // 测试序列化为 TOML 字符串
        let toml_str = toml::to_string_pretty(&config).expect("Failed to serialize config");
        // 测试反序列化
        let loaded_config: AppConfig =
            toml::from_str(&toml_str).expect("Failed to deserialize config");

        assert_eq!(config.database.host, loaded_config.database.host);
        assert_eq!(config.database.port, loaded_config.database.port);
        assert_eq!(config.database.database, loaded_config.database.database);
        assert_eq!(config.database.username, loaded_config.database.username);
        assert_eq!(config.database.password, loaded_config.database.password);
        assert!(loaded_config.csaf_db.is_none());
        assert_eq!(config.logging.level, loaded_config.logging.level);
        assert_eq!(config.logging.file, loaded_config.logging.file);
        assert_eq!(config.logging.stdout, loaded_config.logging.stdout);
        assert_eq!(config.api.group_name, loaded_config.api.group_name);
    }

    #[test]
    fn test_default_logging_config() {
        let default_logging = LoggingConfig::default();
        assert_eq!(default_logging.level, "info");
        assert_eq!(default_logging.file, "");
        // 默认情况下stdout为false，但在from_file中会根据file是否为空来设置
    }

    #[test]
    fn test_default_app_config() {
        let default_config = AppConfig::default();
        assert_eq!(default_config.database.host, "localhost");
        assert_eq!(default_config.database.port, 5432);
        assert_eq!(default_config.database.database, "cu_scanner");
        assert_eq!(default_config.database.username, "user");
        assert_eq!(default_config.database.password, "password");
        assert!(default_config.csaf_db.is_none());

        let default_logging = default_config.logging;
        assert_eq!(default_logging.level, "info");
        assert_eq!(default_logging.file, "");
        assert_eq!(default_logging.stdout, false);

        let default_api = default_config.api;
        assert_eq!(default_api.group_name, "api");
    }

    #[test]
    fn test_csaf_db_config() {
        let config = AppConfig {
            database: DatabaseConfig {
                host: "localhost".to_string(),
                port: 5432,
                database: "cu_scanner".to_string(),
                username: "user".to_string(),
                password: "pass".to_string(),
            },
            csaf_db: Some(DatabaseConfig {
                host: "csaf-host".to_string(),
                port: 5433,
                database: "csaf_database".to_string(),
                username: "csaf_user".to_string(),
                password: "csaf_pass".to_string(),
            }),
            csaf_url: None,
            logging: LoggingConfig::default(),
            api: ApiConfig::default(),
            server: ServerConfig::default(),
            package: PackageConfig::default(),
        };

        // 测试序列化
        let toml_str = toml::to_string_pretty(&config).expect("Failed to serialize config");
        // 测试反序列化
        let loaded_config: AppConfig =
            toml::from_str(&toml_str).expect("Failed to deserialize config");

        assert!(loaded_config.csaf_db.is_some());
        let csaf_db = loaded_config.csaf_db.unwrap();
        assert_eq!(csaf_db.host, "csaf-host");
        assert_eq!(csaf_db.port, 5433);
        assert_eq!(csaf_db.database, "csaf_database");
        assert_eq!(csaf_db.username, "csaf_user");
        assert_eq!(csaf_db.password, "csaf_pass");
    }

    #[test]
    fn test_csaf_url_config() {
        let config = AppConfig {
            database: DatabaseConfig {
                host: "localhost".to_string(),
                port: 5432,
                database: "cu_scanner".to_string(),
                username: "user".to_string(),
                password: "pass".to_string(),
            },
            csaf_db: None,
            csaf_url: Some(CsafUrlConfig {
                url: "https://dl-cdn.openeuler.openatom.cn/security/data/csaf/advisories/index.txt"
                    .to_string(),
                fetch_interval_secs: 3600,
            }),
            logging: LoggingConfig::default(),
            api: ApiConfig::default(),
            server: ServerConfig::default(),
            package: PackageConfig::default(),
        };

        // 测试序列化
        let toml_str = toml::to_string_pretty(&config).expect("Failed to serialize config");
        // 测试反序列化
        let loaded_config: AppConfig =
            toml::from_str(&toml_str).expect("Failed to deserialize config");

        assert!(loaded_config.csaf_url.is_some());
        let csaf_url = loaded_config.csaf_url.unwrap();
        assert_eq!(
            csaf_url.url,
            "https://dl-cdn.openeuler.openatom.cn/security/data/csaf/advisories/index.txt"
        );
        assert_eq!(csaf_url.fetch_interval_secs, 3600);
    }
}
