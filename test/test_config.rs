//! 测试配置管理模块
//! 用于读取和管理测试文件的配置

use std::fs;
use toml::Value;

/// 测试配置结构体
#[derive(Debug, Clone)]
pub struct TestConfig {
    /// CSAF模块测试文件配置
    pub csaf_files: Vec<String>,
    /// Parser模块测试文件配置
    pub parser_files: Vec<String>,
    /// 通用测试文件配置
    pub common_files: Vec<String>,
}

impl TestConfig {
    /// 创建新的测试配置实例
    pub fn new() -> Self {
        Self {
            csaf_files: Vec::new(),
            parser_files: Vec::new(),
            common_files: Vec::new(),
        }
    }

    /// 从TOML配置文件加载配置
    pub fn load_from_file(config_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let config_content = fs::read_to_string(config_path)?;
        let config_value: Value = config_content.parse()?;

        let mut config = TestConfig::new();

        // 读取CSAF文件配置
        if let Some(csaf_section) = config_value.get("csaf") {
            if let Some(files_array) = csaf_section.get("files") {
                if let Some(files) = files_array.as_array() {
                    config.csaf_files = files
                        .iter()
                        .filter_map(|v| v.as_str())
                        .map(|s| s.to_string())
                        .collect();
                }
            }
        }

        // 读取Parser文件配置
        if let Some(parser_section) = config_value.get("parser") {
            if let Some(files_array) = parser_section.get("files") {
                if let Some(files) = files_array.as_array() {
                    config.parser_files = files
                        .iter()
                        .filter_map(|v| v.as_str())
                        .map(|s| s.to_string())
                        .collect();
                }
            }
        }

        // 读取通用文件配置
        if let Some(common_section) = config_value.get("common") {
            if let Some(files_array) = common_section.get("files") {
                if let Some(files) = files_array.as_array() {
                    config.common_files = files
                        .iter()
                        .filter_map(|v| v.as_str())
                        .map(|s| s.to_string())
                        .collect();
                }
            }
        }

        Ok(config)
    }

    /// 获取CSAF测试文件列表
    pub fn get_csaf_files(&self) -> &[String] {
        &self.csaf_files
    }

    /// 获取Parser测试文件列表
    pub fn get_parser_files(&self) -> &[String] {
        &self.parser_files
    }

    /// 获取通用测试文件列表
    pub fn get_common_files(&self) -> &[String] {
        &self.common_files
    }

    /// 根据模块名获取测试文件列表
    pub fn get_files_by_module(&self, module: &str) -> &[String] {
        match module {
            "csaf" => self.get_csaf_files(),
            "parser" => self.get_parser_files(),
            _ => self.get_common_files(),
        }
    }
}

impl Default for TestConfig {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_loading() {
        let config = TestConfig::load_from_file("test_config.toml");
        assert!(config.is_ok());

        let config = config.unwrap();
        assert!(!config.csaf_files.is_empty());
        assert!(!config.parser_files.is_empty());
    }

    #[test]
    fn test_get_files_by_module() {
        let config = TestConfig::load_from_file("test_config.toml").unwrap();

        let csaf_files = config.get_files_by_module("csaf");
        assert!(!csaf_files.is_empty());

        let parser_files = config.get_files_by_module("parser");
        assert!(!parser_files.is_empty());
    }
}
