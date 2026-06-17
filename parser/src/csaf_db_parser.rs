//! CSAF数据库解析器
//!
//! 该模块提供了从CSAF数据库中提取数据并转换为OVAL格式的功能。

use chrono::Utc;
use csaf_database::{CsafQuery, CveInfo, DatabaseConfig, DatabaseManager};
use log::{debug, info};
use oval::{Advisory, Affected, CVE, Definition, Issued, Metadata, OvalDefinitions, Updated};
use regex::Regex;
use std::error::Error;

/// 从SA ID中提取最后两段数字并去掉减号作为OVAL定义ID
fn extract_oval_id_from_sa_id(sa_id: &str) -> String {
    // 使用正则表达式提取最后两段数字
    let re = Regex::new(r"(\d+)-(\d+)$").unwrap();
    if let Some(captures) = re.captures(sa_id) {
        if captures.len() >= 3 {
            // 提取两段数字并去掉减号
            let first_part = &captures[1];
            let second_part = &captures[2];
            return format!("{}{}", first_part, second_part);
        }
    }
    // 如果无法匹配，则返回原始ID
    sa_id.to_string()
}

/// 从CSAF数据库中提取数据并填充到OVAL结构体中
pub async fn parse_csaf_database_to_oval(
    db_config: &DatabaseConfig,
) -> Result<OvalDefinitions, Box<dyn Error>> {
    info!("开始从CSAF数据库解析数据到OVAL格式");

    // 连接数据库
    let db_manager = DatabaseManager::new(db_config).await?;
    let csaf_query = CsafQuery::new(db_manager).await?;
    todo!();
}

/// 根据 SA ID 从数据库查询信息并创建 OVAL 定义
///
/// # 参数
///
/// * `db_config` - 数据库配置
/// * `sa_id` - 安全公告 ID
///
/// # 返回值
///
/// 返回 Result<Definition>，成功时包含 OVAL 定义，失败时包含错误信息
pub async fn create_definition_from_sa_id(
    db_config: &DatabaseConfig,
    sa_id: &str,
) -> Result<Definition, Box<dyn Error>> {
    todo!()
}

/// 根据CVE信息创建CVE条目
#[allow(dead_code)]
fn create_cve_from_cve_info(cve_info: &CveInfo) -> CVE {
    todo!()
}

/// 从数据库中获取某个更新时间之后的所有SA ID
///
/// # 参数
///
/// * `db_config` - 数据库配置
/// * `timestamp` - 时间戳，格式为 "YYYY-MM-DD HH:MM:SS" 或 "YYYY-MM-DD"
///
/// # 返回值
///
/// 返回Result<Vec<String>>\uff0c成功时包含SA ID列表\uff0c失败时包含错误信息
pub async fn get_sa_ids_after_updated_time(
    db_config: &DatabaseConfig,
    timestamp: &str,
) -> Result<Vec<String>, Box<dyn Error>> {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_oval_id_from_sa_id() {
        todo!()
    }
}
