//! Epoch 数据处理模块
//!
//! 提供 RPM 包 epoch 信息的 JSON 解析和序列化功能

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::process::Command;
use log::{debug, warn};
use crate::config::AppConfig;

/// RPM 包 epoch 信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageEpoch {
    /// 包名
    pub name: String,
    /// epoch 值
    pub epoch: u32,
}

/// 包含多个包 epoch 信息的容器
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageEpochs {
    /// 包列表
    pub packages: Vec<PackageEpoch>,
}

impl PackageEpochs {
    /// 创建新的 PackageEpochs 实例
    pub fn new() -> Self {
        PackageEpochs {
            packages: Vec::new(),
        }
    }

    /// 从配置中指定的 JSON 文件加载数据
    ///
    /// # 参数
    /// * `config` - 应用配置
    ///
    /// # 返回值
    /// 返回Result<PackageEpochs, Box<dyn std::error::Error>>，成功时包含加载的数据，失败时包含错误信息
    pub fn from_config(config: &AppConfig) -> Result<Self, Box<dyn std::error::Error>> {
        Self::from_json_file(&config.package.epoch_file)
    }

    /// 从 JSON 文件加载数据
    pub fn from_json_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let epochs: PackageEpochs = serde_json::from_str(&contents)?;
        Ok(epochs)
    }

    /// 保存数据到 JSON 文件
    pub fn to_json_file<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(self)?;
        let mut file = File::create(path)?;
        file.write_all(json.as_bytes())?;
        Ok(())
    }

    /// 从 JSON 字符串加载数据
    pub fn from_json_str(json_str: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let epochs: PackageEpochs = serde_json::from_str(json_str)?;
        Ok(epochs)
    }

    /// 转换为 JSON 字符串
    pub fn to_json_str(&self) -> Result<String, Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(self)?;
        Ok(json)
    }

    /// 获取指定包名的 epoch 值
    pub fn get_epoch(&self, package_name: &str) -> Option<u32> {
        self.packages
            .iter()
            .find(|pkg| pkg.name == package_name)
            .map(|pkg| pkg.epoch)
    }

    /// 从 YUM 查询包的 epoch 值
    ///
    /// # 参数
    /// * `package_name` - RPM 包名
    ///
    /// # 返回值
    /// 返回 Option<u32>，成功时包含 epoch 值，失败时返回 None
    fn get_epoch_from_yum(package_name: &str) -> Option<u32> {
        debug!("尝试从 YUM 查询包 {} 的 epoch", package_name);

        let output = Command::new("yum")
            .args(&["info", package_name])
            .output();

        match output {
            Ok(output) if output.status.success() => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                // 解析 YUM 输出，查找 Epoch 字段
                for line in stdout.lines() {
                    if line.trim().starts_with("Epoch") {
                        if let Some(epoch_str) = line.split(':').nth(1) {
                            if let Ok(epoch) = epoch_str.trim().parse::<u32>() {
                                debug!("从 YUM 查询到包 {} 的 epoch: {}", package_name, epoch);
                                return Some(epoch);
                            }
                        }
                    }
                }
                debug!("YUM 未返回包 {} 的 epoch 信息", package_name);
                None
            }
            Ok(_) => {
                warn!("YUM 查询包 {} 失败", package_name);
                None
            }
            Err(e) => {
                warn!("执行 YUM 命令失败: {}", e);
                None
            }
        }
    }

    /// 从 Extra YUM 源查询包的 epoch 值
    ///
    /// # 参数
    /// * `package_name` - RPM 包名
    ///
    /// # 返回值
    /// 返回 Option<u32>，成功时包含 epoch 值，失败时返回 None
    fn get_epoch_from_extra_yum(package_name: &str) -> Option<u32> {
        debug!("尝试从 Extra YUM 查询包 {} 的 epoch", package_name);

        // 使用 --enablerepo 参数启用额外的仓库
        let output = Command::new("yum")
            .args(&["info", "--enablerepo=*", package_name])
            .output();

        match output {
            Ok(output) if output.status.success() => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                // 解析 YUM 输出，查找 Epoch 字段
                for line in stdout.lines() {
                    if line.trim().starts_with("Epoch") {
                        if let Some(epoch_str) = line.split(':').nth(1) {
                            if let Ok(epoch) = epoch_str.trim().parse::<u32>() {
                                debug!("从 Extra YUM 查询到包 {} 的 epoch: {}", package_name, epoch);
                                return Some(epoch);
                            }
                        }
                    }
                }
                debug!("Extra YUM 未返回包 {} 的 epoch 信息", package_name);
                None
            }
            Ok(_) => {
                warn!("Extra YUM 查询包 {} 失败", package_name);
                None
            }
            Err(e) => {
                warn!("执行 Extra YUM 命令失败: {}", e);
                None
            }
        }
    }

    /// 根据配置和优先级获取指定包名的 epoch 值
    ///
    /// 优先级顺序:
    /// 1. Extra YUM (如果配置启用)
    /// 2. YUM (如果配置启用)
    /// 3. JSON 文件
    /// 4. 默认值 0
    ///
    /// # 参数
    /// * `package_name` - RPM 包名
    /// * `config` - 应用配置
    ///
    /// # 返回值
    /// 返回 u32 类型的 epoch 值，如果所有来源都未找到则返回 0
    pub fn get_epoch_with_priority(&self, package_name: &str, config: &AppConfig) -> u32 {
        // 优先级 1: Extra YUM
        if config.package.use_extra_yum {
            if let Some(epoch) = Self::get_epoch_from_extra_yum(package_name) {
                debug!("从 Extra YUM 获取到包 {} 的 epoch: {}", package_name, epoch);
                return epoch;
            }
        }

        // 优先级 2: YUM
        if config.package.use_yum {
            if let Some(epoch) = Self::get_epoch_from_yum(package_name) {
                debug!("从 YUM 获取到包 {} 的 epoch: {}", package_name, epoch);
                return epoch;
            }
        }

        // 优先级 3: JSON 文件
        if let Some(epoch) = self.get_epoch(package_name) {
            debug!("从 JSON 文件获取到包 {} 的 epoch: {}", package_name, epoch);
            return epoch;
        }

        // 优先级 4: 默认值 0
        debug!("包 {} 未找到 epoch 信息，使用默认值 0", package_name);
        0
    }

    /// 添加或更新包的 epoch 信息
    pub fn set_epoch(&mut self, package_name: &str, epoch: u32) {
        if let Some(pkg) = self.packages.iter_mut().find(|pkg| pkg.name == package_name) {
            pkg.epoch = epoch;
        } else {
            self.packages.push(PackageEpoch {
                name: package_name.to_string(),
                epoch,
            });
        }
    }

    /// 转换为 HashMap 便于快速查找
    pub fn to_hashmap(&self) -> HashMap<String, u32> {
        let mut map = HashMap::new();
        for pkg in &self.packages {
            map.insert(pkg.name.clone(), pkg.epoch);
        }
        map
    }
}

impl Default for PackageEpochs {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_package_epochs_new() {
        let epochs = PackageEpochs::new();
        assert!(epochs.packages.is_empty());
    }

    #[test]
    fn test_package_epochs_default() {
        let epochs = PackageEpochs::default();
        assert!(epochs.packages.is_empty());
    }

    #[test]
    fn test_add_and_get_epoch() {
        let mut epochs = PackageEpochs::new();
        epochs.set_epoch("test-package", 42);

        assert_eq!(epochs.get_epoch("test-package"), Some(42));
        assert_eq!(epochs.get_epoch("non-existent"), None);
    }

    #[test]
    fn test_update_epoch() {
        let mut epochs = PackageEpochs::new();
        epochs.set_epoch("test-package", 42);
        epochs.set_epoch("test-package", 84);

        assert_eq!(epochs.get_epoch("test-package"), Some(84));
    }

    #[test]
    fn test_to_hashmap() {
        let mut epochs = PackageEpochs::new();
        epochs.set_epoch("package1", 1);
        epochs.set_epoch("package2", 2);

        let map = epochs.to_hashmap();
        assert_eq!(map.get("package1"), Some(&1));
        assert_eq!(map.get("package2"), Some(&2));
    }

    #[test]
    fn test_json_serialization() {
        let json_str = r#"{
  "packages": [
    {
      "name": "bind-utils",
      "epoch": 32
    },
    {
      "name": "grub2-pc",
      "epoch": 1
    },
    {
      "name": "irqbalance",
      "epoch": 3
    },
    {
      "name": "NetworkManager",
      "epoch": 1
    },
    {
      "name": "lvm2",
      "epoch": 8
    },
    {
      "name": "tcpdump",
      "epoch": 14
    },
    {
      "name": "audit",
      "epoch": 1
    }
  ]
}"#;

        let epochs = PackageEpochs::from_json_str(json_str).unwrap();
        assert_eq!(epochs.packages.len(), 7);
        assert_eq!(epochs.get_epoch("bind-utils"), Some(32));
        assert_eq!(epochs.get_epoch("grub2-pc"), Some(1));
        assert_eq!(epochs.get_epoch("irqbalance"), Some(3));
        assert_eq!(epochs.get_epoch("NetworkManager"), Some(1));
        assert_eq!(epochs.get_epoch("lvm2"), Some(8));
        assert_eq!(epochs.get_epoch("tcpdump"), Some(14));
        assert_eq!(epochs.get_epoch("audit"), Some(1));

        let serialized = epochs.to_json_str().unwrap();
        let deserialized = PackageEpochs::from_json_str(&serialized).unwrap();
        assert_eq!(epochs.packages.len(), deserialized.packages.len());
    }
}
