//! 数据库实体模块
//!
//! 该模块提供了数据库实体结构体的定义。

use serde::{Deserialize, Serialize};

// 简化版数据库实体结构体定义

/// 操作系统信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OsInfo {
    pub id: Option<i64>, // 数据库自增ID
    pub os_type: String,
    pub os_version: String,
    pub package_name: String,
    pub verify_file: String,
    pub verify_pattern: String,
    pub dist: String,
    pub description: Option<String>,
}

/// OVAL定义信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OvalDefinition {
    pub id: String,
    pub class: String,
    pub version: u32,
    pub title: String,
    pub description: String,
    pub family: String,
    pub platform: String,
    pub severity: String,
    pub rights: String,
    pub from: String,
    pub issued_date: String,
    pub updated_date: String,
    pub os_info_id: Option<i64>, // 操作系统信息ID
}

/// 引用信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reference {
    pub ref_id: String,
    pub ref_url: String,
    pub source: String,
}

/// CVE信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cve {
    pub cve_id: String,
    pub cvss3: String,
    pub impact: String,
    pub href: String,
    pub content: String,
}

/// 条件信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Criterion {
    pub comment: String,
    pub test_ref: String,
}

/// 条件标准信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Criteria {
    pub operator: String,
    pub criterion: Vec<Criterion>,
    pub sub_criteria: Option<Vec<Criteria>>,
}

/// RPM信息测试
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpmInfoTest {
    pub check: String,
    pub comment: String,
    pub test_id: String,
    pub version: u32,
    pub object_ref: String,
    pub state_ref: String,
}

/// RPM信息对象
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpmInfoObject {
    pub id: Option<i64>, // 数据库自增ID
    pub object_id: String,
    pub ver: u64,
    pub rpm_name: String,
}

/// RPM信息状态（合并EVR信息）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpmInfoState {
    pub state_id: String,
    pub version: String,
    // EVR信息直接嵌入
    pub evr_datatype: Option<String>,
    pub evr_operation: Option<String>,
    pub evr_value: Option<String>,
}
