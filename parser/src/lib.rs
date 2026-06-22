//! CSAF到OVAL格式转换工具
//!
//! 该模块提供了将CSAF(Common Security Advisory Framework)格式的安全公告
//! 转换为OVAL(Open Vulnerability and Assessment Language)格式的功能。
//!
//! # 使用示例
//!
//! ## 单文件转换
//! ```no_run
//! use parser::csaf_to_oval;
//! use csaf::CSAF;
//!
//! let csaf = CSAF::from_file("advisory.json").unwrap();
//! let oval = csaf_to_oval(&csaf).unwrap();
//! oval.save_to_file("output.xml").unwrap();
//! ```
//!
//! ## 批量转换（共享IdGenerator确保批次内ID不重复）
//! ```no_run
//! use parser::batch_csaf_to_oval;
//! use csaf::CSAF;
//!
//! let csaf1 = CSAF::from_file("advisory1.json").unwrap();
//! let csaf2 = CSAF::from_file("advisory2.json").unwrap();
//! let csaf_list = vec![&csaf1, &csaf2];
//!
//! let oval_list = batch_csaf_to_oval(&csaf_list, 10000).unwrap();
//! for (i, oval) in oval_list.iter().enumerate() {
//!     oval.save_to_file(&format!("output_{}.xml", i)).unwrap();
//! }
//! ```
//!
//! ## 手动控制共享IdGenerator
//! ```no_run
//! use parser::{csaf_to_oval_with_shared_generator, IdGenerator};
//! use csaf::CSAF;
//!
//! let mut shared_gen = IdGenerator::new(10000);
//!
//! let csaf1 = CSAF::from_file("advisory1.json").unwrap();
//! let oval1 = csaf_to_oval_with_shared_generator(&csaf1, &mut shared_gen).unwrap();
//!
//! let csaf2 = CSAF::from_file("advisory2.json").unwrap();
//! let oval2 = csaf_to_oval_with_shared_generator(&csaf2, &mut shared_gen).unwrap();
//! // oval1 和 oval2 的 ID 不会重复
//! ```

pub mod csaf_db_parser;

use chrono::{DateTime, Utc};
use csaf::{CSAF, Vulnerabilitie};
use log::{debug, error, info, warn};
use oval::{
    Affected, CVE, Criteria, Criterion, Definition, Metadata, ObjectReference, OvalDefinitions,
    RpmInfoObject, RpmInfoState, RpmInfoTest, StateReference,
};
use std::collections::HashMap;
use utils::{Result, id_counter::IdCounterManager};

/// OVAL 构建结果类型别名
/// 包含构建的检查条件和相关的测试、对象、状态列表
type OvalCriteriaResult = (
    Criteria,                           // 检查条件
    Vec<RpmInfoTest>,                   // RPM信息测试列表
    Vec<RpmInfoObject>,                 // RPM信息对象列表
    Vec<RpmInfoState>,                  // RPM信息状态列表
    Vec<oval::RpmVerifyFileTest>,       // RPM验证文件测试列表
    Vec<oval::RpmVerifyFileObject>,     // RPM验证文件对象列表
    Vec<oval::RpmVerifyFileState>,      // RPM验证文件状态列表
);

/// 操作系统信息（用于OVAL转换，不依赖数据库）
#[derive(Debug, Clone)]
pub struct OsInfo {
    pub os_type: String,
    pub os_version: String,
    pub package_name: String,
    pub verify_file: String,
    pub verify_pattern: String,
    pub dist: String,
    pub description: String,
}

/// 根据 dist 获取对应的 os_info_id
/// 这个映射必须与 database/src/schema.rs 中的 init_os_info_data 保持一致
fn get_os_info_id_by_dist(dist: &str) -> i64 {
    match dist {
        "oe1" => 1,    // openEuler 20.03
        "oe2203" => 2, // openEuler 22.03
        "oe2403" => 3, // openEuler 24.03
        "el7" => 4,    // Red Hat Enterprise Linux 7
        "el9" => 5,    // Red Hat Enterprise Linux 9
        "el8" => 6,    // Red Hat Enterprise Linux 8
        "ule4" => 7,   // China Unicom Linux 4
        _ => 0,        // Unknown
    }
}

/// 根据 os_info_id 生成操作系统检查相关的固定 ID
/// 这个函数必须与 database/src/lib.rs 中的 generate_os_check_ids 保持一致
///
/// # ID 分配策略
///
/// - 每个 OS 占用 10 个 ID 空间
/// - base_id = os_info_id * 10
/// - object: base_id + 0
/// - state_full (name + version): base_id + 1
/// - state_name_only (仅 name): base_id + 2
/// - test (must be installed): base_id + 3
/// - test (is installed): base_id + 4
fn generate_os_check_ids(os_info_id: i64) -> (String, String, String, String, String) {
    let base_id = os_info_id * 10;

    let os_object_id = format!("{}{}", oval::CU_LINUX_SA_OBJ_PREFIX, base_id);
    let os_state_full_id = format!("{}{}", oval::CU_LINUX_SA_STE_PREFIX, base_id + 1);
    let os_state_name_only_id = format!("{}{}", oval::CU_LINUX_SA_STE_PREFIX, base_id + 2);
    let os_test_must_id = format!("{}{}", oval::CU_LINUX_SA_TST_PREFIX, base_id + 3);
    let os_test_is_id = format!("{}{}", oval::CU_LINUX_SA_TST_PREFIX, base_id + 4);

    (
        os_object_id,
        os_state_full_id,
        os_state_name_only_id,
        os_test_must_id,
        os_test_is_id,
    )
}

/// 获取预定义的操作系统信息列表
fn get_predefined_os_info() -> Vec<OsInfo> {
    vec![
        OsInfo {
            os_type: "openEuler".to_string(),
            os_version: "20.03".to_string(),
            package_name: "openeuler-release".to_string(),
            verify_file: "/etc/os-release".to_string(),
            verify_pattern: "^openEuler".to_string(),
            dist: "oe1".to_string(),
            description: "openEuler 20.03 LTS".to_string(),
        },
        OsInfo {
            os_type: "openEuler".to_string(),
            os_version: "22.03".to_string(),
            package_name: "openeuler-release".to_string(),
            verify_file: "/etc/os-release".to_string(),
            verify_pattern: "^openEuler".to_string(),
            dist: "oe2203".to_string(),
            description: "openEuler 22.03 LTS".to_string(),
        },
        OsInfo {
            os_type: "openEuler".to_string(),
            os_version: "24.03".to_string(),
            package_name: "openeuler-release".to_string(),
            verify_file: "/etc/os-release".to_string(),
            verify_pattern: "^openEuler".to_string(),
            dist: "oe2403".to_string(),
            description: "openEuler 24.03 LTS".to_string(),
        },
        OsInfo {
            os_type: "Red Hat Enterprise Linux".to_string(),
            os_version: "7".to_string(),
            package_name: "redhat-release".to_string(),
            verify_file: "/etc/redhat-release".to_string(),
            verify_pattern: "^Red Hat Enterprise Linux".to_string(),
            dist: "el7".to_string(),
            description: "Red Hat Enterprise Linux 7".to_string(),
        },
        OsInfo {
            os_type: "Red Hat Enterprise Linux".to_string(),
            os_version: "8".to_string(),
            package_name: "redhat-release".to_string(),
            verify_file: "/etc/redhat-release".to_string(),
            verify_pattern: "^Red Hat Enterprise Linux".to_string(),
            dist: "el8".to_string(),
            description: "Red Hat Enterprise Linux 8".to_string(),
        },
        OsInfo {
            os_type: "China Unicom Linux".to_string(),
            os_version: "4".to_string(),
            package_name: "culinux-release".to_string(),
            verify_file: "/etc/os-release".to_string(),
            verify_pattern: "^CULinux Enterprise Edition".to_string(),
            dist: "ule4".to_string(),
            description: "China Unicom Linux 4.0".to_string(),
        },
    ]
}
/// 从软件包版本中提取dist标识
/// 例如: "ansible-2.9-1.oe1" -> Some("oe1")
///      "package-1.0-1.el7" -> Some("el7")
fn extract_dist_from_package(package_version: &str) -> Option<String> {
    // 从预定义的OS信息中获取所有dist标识
    let all_dists: Vec<String> = get_predefined_os_info()
        .iter()
        .map(|os| os.dist.clone())
        .collect();
    // 按长度降序排序，优先匹配更长的dist（如oe2403优先于oe1）
    let mut sorted_dists = all_dists.clone();
    sorted_dists.sort_by_key(|b| std::cmp::Reverse(b.len()));
    // 精确匹配
    for dist in &sorted_dists {
        if package_version.contains(dist) {
            debug!(
                "从软件包版本 {} 中精确匹配到dist: {}",
                package_version, dist
            );
            return Some(dist.clone());
        }
    }
    // 模糊匹配：oe2003sp4 -> oe1 (openEuler 20.03系列)
    if package_version.contains("oe2003") || package_version.contains("oe20.03") {
        debug!(
            "从软件包版本 {} 中模糊匹配到dist: oe1 (openEuler 20.03)",
            package_version
        );
        return Some("oe1".to_string());
    }
    // 模糊匹配：oe2203sp* -> oe2203
    if package_version.contains("oe2203") || package_version.contains("oe22.03") {
        debug!(
            "从软件包版本 {} 中模糊匹配到dist: oe2203 (openEuler 22.03)",
            package_version
        );
        return Some("oe2203".to_string());
    }
    // 模糊匹配：oe2403sp* -> oe2403
    if package_version.contains("oe2403") || package_version.contains("oe24.03") {
        debug!(
            "从软件包版本 {} 中模糊匹配到dist: oe2403 (openEuler 24.03)",
            package_version
        );
        return Some("oe2403".to_string());
    }
    warn!("无法从软件包版本 {} 中提取dist标识", package_version);
    None
}
/// 根据dist标识匹配操作系统信息
/// 如果无法匹配，返回"Unknown OS"
fn match_os_info_by_dist(dist: Option<&str>) -> OsInfo {
    let predefined = get_predefined_os_info();
    if let Some(dist_val) = dist {
        for os_info in predefined {
            if os_info.dist == dist_val {
                debug!("匹配到OS信息: {} {}", os_info.os_type, os_info.os_version);
                return os_info;
            }
        }
    }
    warn!("无法匹配OS信息，使用Unknown OS");
    OsInfo {
        os_type: "Unknown OS".to_string(),
        os_version: "Unknown".to_string(),
        package_name: "unknown-release".to_string(),
        verify_file: "/etc/unknown-release".to_string(),
        verify_pattern: "^Unknown".to_string(),
        dist: dist.unwrap_or("unknown").to_string(),
        description: "Unknown Operating System".to_string(),
    }
}
#[derive(Debug)]
pub struct IdGenerator {
    /// 对象ID映射，确保相同对象使用相同ID
    object_ids: HashMap<String, String>,
    /// 状态ID映射，确保相同状态使用相同ID
    state_ids: HashMap<String, String>,
    /// 测试ID映射，确保相同测试使用相同ID
    test_ids: HashMap<String, String>,
    /// 定义ID映射，确保相同定义使用相同ID
    definition_ids: HashMap<String, String>,
    /// ID计数器管理器
    id_counter: IdCounterManager,
}

impl IdGenerator {
    /// 创建新的ID生成器
    pub fn new(initial_counter: u64) -> Self {
        info!("创建新的ID生成器，初始计数器值: {}", initial_counter);
        Self {
            object_ids: HashMap::new(),
            state_ids: HashMap::new(),
            test_ids: HashMap::new(),
            definition_ids: HashMap::new(),
            id_counter: IdCounterManager::new(initial_counter),
        }
    }

    /// 生成唯一ID
    fn generate_unique_id(&mut self, prefix: &str) -> String {
        let id = self.id_counter.generate_unique_id(prefix);
        debug!("生成唯一ID: {}{}", prefix, id);
        id
    }

    /// 获取当前计数器值
    pub fn get_current_counter(&self) -> u64 {
        self.id_counter.get_current_counter()
    }

    /// 设置当前计数器值
    pub fn set_current_counter(&mut self, counter: u64) {
        self.id_counter.set_current_counter(counter);
    }

    /// 获取或创建对象ID，确保相同对象名使用相同ID
    pub fn get_or_create_object_id(&mut self, object_name: &str, prefix: &str) -> String {
        if let Some(id) = self.object_ids.get(object_name) {
            debug!("使用现有对象ID: {} -> {}", object_name, id);
            id.clone()
        } else {
            let id = self.generate_unique_id(prefix);
            self.object_ids.insert(object_name.to_string(), id.clone());
            debug!("创建新对象ID: {} -> {}", object_name, id);
            id
        }
    }

    /// 获取或创建状态ID，确保相同EVR使用相同ID
    pub fn get_or_create_state_id(&mut self, evr: &str, prefix: &str) -> String {
        if let Some(id) = self.state_ids.get(evr) {
            debug!("使用现有状态ID: {} -> {}", evr, id);
            id.clone()
        } else {
            let id = self.generate_unique_id(prefix);
            self.state_ids.insert(evr.to_string(), id.clone());
            debug!("创建新状态ID: {} -> {}", evr, id);
            id
        }
    }

    /// 获取或创建测试ID，确保相同测试使用相同ID
    pub fn get_or_create_test_id(&mut self, test_key: &str, prefix: &str) -> String {
        if let Some(id) = self.test_ids.get(test_key) {
            debug!("使用现有测试ID: {} -> {}", test_key, id);
            id.clone()
        } else {
            let id = self.generate_unique_id(prefix);
            self.test_ids.insert(test_key.to_string(), id.clone());
            debug!("创建新测试ID: {} -> {}", test_key, id);
            id
        }
    }

    /// 获取或创建定义ID，确保相同定义使用相同ID
    pub fn get_or_create_definition_id(&mut self, definition_key: &str, prefix: &str) -> String {
        if let Some(id) = self.definition_ids.get(definition_key) {
            debug!("使用现有定义ID: {} -> {}", definition_key, id);
            id.clone()
        } else {
            let id = self.generate_unique_id(prefix);
            self.definition_ids
                .insert(definition_key.to_string(), id.clone());
            debug!("创建新定义ID: {} -> {}", definition_key, id);
            id
        }
    }

    /// 为CSAF漏洞生成定义ID
    pub fn generate_definition_id_for_cve(&mut self, cve_id: &str) -> String {
        let key = format!("cve:{}", cve_id);
        let id = self.get_or_create_definition_id(&key, oval::CU_LINUX_SA_DEF_PREFIX);
        debug!("为CVE生成定义ID: {} -> {}", cve_id, id);
        id
    }

    /// 为软件包生成对象ID
    pub fn generate_object_id_for_package(&mut self, package_name: &str) -> String {
        todo!()
    }

    /// 为EVR生成状态ID
    pub fn generate_state_id_for_evr(&mut self, evr: &str) -> String {
        todo!()
    }

    /// 为测试生成ID
    pub fn generate_test_id(&mut self, package_name: &str, evr: &str) -> String {
        todo!()
    }

    /// 为基本测试生成ID（用于OS检查等）
    pub fn generate_base_test_id(&mut self, test_type: &str) -> String {
        todo!()
    }
}

impl Default for IdGenerator {
    fn default() -> Self {
        todo!()
    }
}

/// 将CSAF格式转换为OVAL格式（使用共享IdGenerator）
///
/// # 参数
///
/// * `csaf` - CSAF结构体的引用
/// * `id_generator` - 共享的ID生成器可变引用
///
/// # 返回值
///
/// 返回Result<OvalDefinitions>，成功时包含转换后的OVAL定义，失败时包含错误信息
pub fn csaf_to_oval_with_shared_generator(
    csaf: &CSAF,
    id_generator: &mut IdGenerator,
) -> Result<OvalDefinitions> {
    todo!()
}
/// 将CSAF格式转换为OVAL格式
///
/// # 参数
///
/// * `csaf` - CSAF结构体的引用
/// * `initial_counter` - 初始计数器值，用于确保ID唯一性
///
/// # 返回值
///
/// 返回Result<OvalDefinitions>，成功时包含转换后的OVAL定义，失败时包含错误信息
pub fn csaf_to_oval_with_counter(csaf: &CSAF, initial_counter: u64) -> Result<OvalDefinitions> {
    todo!()
}

/// 将CSAF格式转换为OVAL格式（使用默认计数器）
///
/// # 参数
///
/// * `csaf` - CSAF结构体的引用
///
/// # 返回值
///
/// 返回Result<OvalDefinitions>，成功时包含转换后的OVAL定义，失败时包含错误信息
pub fn csaf_to_oval(csaf: &CSAF) -> Result<OvalDefinitions> {
    todo!()
}
/// 批量将CSAF格式转换为OVAL格式（共享IdGenerator确保批次内ID唯一）
///
/// # 参数
///
/// * `csaf_list` - CSAF结构体引用的切片
/// * `initial_counter` - 初始计数器值
///
/// # 返回值
///
/// 返回Result<Vec<OvalDefinitions>>，成功时包含转换后的OVAL定义列表
pub fn batch_csaf_to_oval(
    csaf_list: &[&CSAF],
    initial_counter: u64,
) -> Result<Vec<OvalDefinitions>> {
    todo!()
}

/// 解析软件包字符串
///
/// # 参数
///
/// * `pkg_string` - 软件包字符串，格式为"os-name-os-version:package-name-version-release"
///
/// # 返回值
///
/// 返回Option<(String, String, String, String)>，包含：
/// 1. 操作系统完整名称
/// 2. 软件包名称
/// 3. EVR (Epoch:Version-Release)
/// 4. 操作系统名称
///
/// 如果解析失败则返回None
pub fn parse_package_string(pkg_string: &str) -> Option<(String, String, String, String)> {
    todo!()
}

/// 填充definition
///
/// # 参数
///
/// * `sa` - CSAF结构体的引用，所有definition中的字段均从此处获取
/// * `definition` - definition的可变引用，此字段作为引用返回的值
///
/// # 返回值
///
/// 返回操作的结果，如果解析失败，则返回Error
pub fn fill_definition(sa: &CSAF, definition: &mut Definition) -> Result<()> {
    todo!()
}

/// 处理CSAF ID，仅保留最后的数字-数字部分，并将减号去掉
///
/// # 参数
///
/// * `id` - 原始CSAF ID
///
/// # 返回值
///
/// 返回处理后的ID字符串
pub fn process_csaf_id(id: &str) -> String {
    todo!()
}

/// 构建OVAL检查条件
///
/// # 参数
///
/// * `sa` - Vulnerabilitie结构体的引用
/// * `id_generator` - ID生成器的可变引用
///
/// # 返回值
///
/// 返回Result<(Criteria, Vec<RpmInfoTest>, Vec<RpmInfoObject>, Vec<RpmInfoState>, Vec<RpmVerifyFileTest>, Vec<RpmVerifyFileObject>, Vec<RpmVerifyFileState>)>，
/// 包含构建的检查条件和相关的测试、对象、状态列表
pub fn build_oval_criteria(
    sa: &Vulnerabilitie,
    id_generator: &mut IdGenerator,
) -> Result<OvalCriteriaResult> {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use csaf::CSAF;
    use std::fs;

    // 从配置文件读取测试文件路径
    fn get_test_file_path(module: &str, filename: &str) -> String {
        todo!()
    }

    #[test]
    fn test_parse_package_string() {
        todo!()
    }

    #[test]
    fn test_parse_package_string_with_complex_name() {
        todo!()
    }

    #[test]
    fn test_parse_package_string_invalid_format() {
        todo!()
    }

    #[test]
    fn test_parse_package_string_no_arch() {
        todo!()
    }

    #[test]
    fn test_id_generator() {
        todo!()
    }

    #[test]
    fn test_id_generator_default() {
        todo!()
    }

    #[test]
    fn test_id_generator_counter_operations() {
        todo!()
    }

    #[test]
    fn test_id_generator_definition_id() {
        todo!()
    }

    #[test]
    fn test_id_generator_base_test_id() {
        todo!()
    }

    #[test]
    fn test_process_csaf_id() {
        todo!()
    }

    #[test]
    fn test_csaf_to_oval_conversion() {
        todo!()
    }

    #[test]
    fn test_csaf_to_oval_with_custom_counter() {
        todo!()
    }

    #[test]
    fn test_fill_definition() {
        todo!()
    }

    #[test]
    fn test_build_oval_criteria() {
        todo!()
    }

    #[test]
    fn test_build_oval_criteria_deduplication() {
        todo!()
    }

    #[test]
    fn test_csaf_to_oval_file_conversion() {
        todo!()
    }

    #[test]
    fn test_oval_xml_structure() {
        todo!()
    }

    #[test]
    fn test_id_generator_prefix_consistency() {
        todo!()
    }
}
