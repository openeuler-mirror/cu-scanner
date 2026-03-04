/// 通用Result返回方式，统一处理
pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T> = std::result::Result<T, Error>;

pub mod config;
pub mod id_counter;
pub mod log;
pub mod epoch;

#[cfg(test)]
mod log_test;

use std::sync::OnceLock;

/// 全局 epoch 数据
static EPOCH_DATA: OnceLock<epoch::PackageEpochs> = OnceLock::new();

/// 全局配置
static GLOBAL_CONFIG: OnceLock<config::AppConfig> = OnceLock::new();

/// 为 EVR 字符串添加 epoch 前缀
///
/// 由于当前从 CSAF 无法直接解析 RPM 的 epoch 字段，
/// 此函数提供统一的 epoch 前缀处理。根据配置和优先级从不同来源查询 epoch 值。
///
/// 优先级顺序:
/// 1. Extra YUM (如果配置启用)
/// 2. YUM (如果配置启用)
/// 3. JSON 文件
/// 4. 默认值 0
///
/// # 参数
/// * `package_name` - RPM 包名（例如: "openssh"）
/// * `version_release` - 版本-发行号字符串（例如: "9.6p1-6.ule4"）
///
/// # 返回值
/// 返回带 epoch 前缀的完整 EVR 字符串（例如: "0:9.6p1-6.ule4"）
///
/// # 示例
/// ```
/// use utils::add_epoch_prefix;
/// let evr = add_epoch_prefix("openssh", "9.6p1-6.ule4");
/// assert_eq!(evr, "0:9.6p1-6.ule4");
/// ```
pub fn add_epoch_prefix(package_name: &str, version_release: &str) -> String {
    // 如果有全局配置和 epoch 数据，使用优先级查询
    if let (Some(config), Some(epochs)) = (GLOBAL_CONFIG.get(), EPOCH_DATA.get()) {
        let epoch = epochs.get_epoch_with_priority(package_name, config);
        return format!("{}:{}", epoch, version_release);
    }

    // 如果只有 epoch 数据没有配置，尝试从 JSON 获取
    if let Some(epochs) = EPOCH_DATA.get() {
        if let Some(epoch) = epochs.get_epoch(package_name) {
            return format!("{}:{}", epoch, version_release);
        }
    }

    // 默认返回 "0:" 作为 epoch 前缀
    format!("0:{}", version_release)
}

/// 设置全局 epoch 数据
///
/// # 参数
/// * `epochs` - PackageEpochs 实例
///
/// # 返回值
/// 如果设置成功返回 Ok(())，如果已经设置过则返回 Err(utils::Error)
pub fn set_global_epoch_data(epochs: epoch::PackageEpochs) -> Result<()> {
    EPOCH_DATA.set(epochs).map_err(|_| "Epoch data already set".into())
}

/// 设置全局配置
///
/// # 参数
/// * `config` - AppConfig 实例
///
/// # 返回值
/// 如果设置成功返回 Ok(())，如果已经设置过则返回 Err(utils::Error)
pub fn set_global_config(config: config::AppConfig) -> Result<()> {
    GLOBAL_CONFIG.set(config).map_err(|_| "Config already set".into())
}
