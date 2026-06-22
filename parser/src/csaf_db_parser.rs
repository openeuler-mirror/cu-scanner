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

    // 创建OVAL定义
    let mut oval_definitions = OvalDefinitions::new();

    // 获取所有安全公告信息
    let sa_infos = csaf_query.get_all_sa_info().await?;
    info!("获取到 {} 条安全公告信息", sa_infos.len());

    // 为每个安全公告创建OVAL定义
    for sa_info in sa_infos {
        let definition = create_definition_from_sa_id(db_config, &sa_info.sa_id).await?;
        oval_definitions.definitions.items.push(definition);
    }

    // 获取所有CVE信息
    let cve_infos = csaf_query.get_all_cve_info().await?;
    info!("获取到 {} 条CVE信息", cve_infos.len());

    // TODO: 根据需要将CVE信息添加到相应的OVAL定义中

    debug!("成功解析CSAF数据库数据到OVAL格式");
    Ok(oval_definitions)
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
    debug!("根据 SA ID {} 从数据库查询信息并创建 OVAL 定义", sa_id);

    // 连接数据库
    let db_manager = DatabaseManager::new(db_config).await?;
    let csaf_query = CsafQuery::new(db_manager).await?;

    // 从数据库查询 SA 信息
    let sa_info = csaf_query.get_sa_info_by_sa_id(sa_id).await?;

    let sa_info = match sa_info {
        Some(info) => info,
        None => {
            return Err(format!("未找到 SA ID: {}", sa_id).into());
        }
    };

    // 从 SA ID 中提取 OVAL 定义 ID
    let oval_id = extract_oval_id_from_sa_id(&sa_info.sa_id);

    let mut oval_definitions = OvalDefinitions::new();
    let now = Utc::now();
    // 使用RFC3339格式（符合xs:dateTime要求）
    let formatted_time = now.to_rfc3339();
    oval_definitions.generator.time_stamp = formatted_time.clone();
    // 创建 OVAL 定义
    let mut definition = Definition::new();
    definition.id = format!("oval:cn.chinaunicom.culinux.cusa:def:{}", oval_id);
    definition.version = 1; // 默认版本

    // 设置定义类别
    definition.class = "patch".to_string();

    // 创建元数据
    let mut metadata = Metadata::new();
    metadata.title = sa_info.synopsis.clone().unwrap_or_default();
    metadata.description = sa_info.description.clone().unwrap_or_default();

    // 设置影响范围
    let mut affected = Affected::new();
    affected.platform = sa_info.summary.clone().unwrap_or_default();
    metadata.affected = affected;

    // 创建建议信息
    let mut advisory = Advisory::new();
    if let Some(ref severity) = sa_info.severity {
        advisory.severity = severity.clone();
    }

    // 设置发布和更新日期
    let mut issued = Issued::new();
    if let Some(ref created_time) = sa_info.created_time {
        issued.date = created_time.clone();
    }
    advisory.issued = issued;

    let mut updated = Updated::new();
    if let Some(ref updated_time) = sa_info.updated_time {
        updated.date = updated_time.clone();
    }
    advisory.updated = updated;

    metadata.advisory = advisory;

    definition.metadata = metadata;

    debug!("成功为安全公告 {} 创建 OVAL 定义", sa_info.sa_id);
    Ok(definition)
}

/// 根据CVE信息创建CVE条目
#[allow(dead_code)]
fn create_cve_from_cve_info(cve_info: &CveInfo) -> CVE {
    debug!("为CVE {} 创建CVE条目", cve_info.cve_id);

    let mut cve = CVE::new();
    cve.content = cve_info.cve_id.clone();
    cve.href = format!(
        "https://cve.mitre.org/cgi-bin/cvename.cgi?name={}",
        cve_info.cve_id
    );
    // 设置默认值
    cve.cvss3 = "".to_string();
    cve.impact = "".to_string();

    debug!("成功为CVE {} 创建CVE条目", cve_info.cve_id);
    cve
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
    info!("开始从 CSAF 数据库查询 {} 之后更新的 SA ID 列表", timestamp);

    // 连接数据库
    let db_manager = DatabaseManager::new(db_config).await?;
    let csaf_query = CsafQuery::new(db_manager).await?;

    // 查询更新时间之后的 SA ID
    let sa_ids = csaf_query.get_sa_ids_after_updated_time(timestamp).await?;

    info!(
        "成功查询到 {} 个更新时间在 {} 之后的 SA ID",
        sa_ids.len(),
        timestamp
    );
    Ok(sa_ids)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_oval_id_from_sa_id() {
        // 测试正常情况
        assert_eq!(extract_oval_id_from_sa_id("SA-2025-1004"), "20251004");
        assert_eq!(extract_oval_id_from_sa_id("CSAF-ID-2025-1004"), "20251004");

        // 测试不匹配的情况
        assert_eq!(extract_oval_id_from_sa_id("invalid-id"), "invalid-id");
        assert_eq!(extract_oval_id_from_sa_id("SA-2025"), "SA-2025");
    }
}
