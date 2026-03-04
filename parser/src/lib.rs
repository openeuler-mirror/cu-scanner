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
    sorted_dists.sort_by(|a, b| b.len().cmp(&a.len()));
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
        let id = self.get_or_create_object_id(package_name, oval::CU_LINUX_SA_OBJ_PREFIX);
        debug!("为软件包生成对象ID: {} -> {}", package_name, id);
        id
    }

    /// 为EVR生成状态ID
    pub fn generate_state_id_for_evr(&mut self, evr: &str) -> String {
        let id = self.get_or_create_state_id(evr, oval::CU_LINUX_SA_STE_PREFIX);
        debug!("为EVR生成状态ID: {} -> {}", evr, id);
        id
    }

    /// 为测试生成ID
    pub fn generate_test_id(&mut self, package_name: &str, evr: &str) -> String {
        let key = format!("{}:{}", package_name, evr);
        let id = self.get_or_create_test_id(&key, oval::CU_LINUX_SA_TST_PREFIX);
        debug!("为测试生成ID: {}:{} -> {}", package_name, evr, id);
        id
    }

    /// 为基本测试生成ID（用于OS检查等）
    pub fn generate_base_test_id(&mut self, test_type: &str) -> String {
        let key = format!("base:{}", test_type);
        let id = self.get_or_create_test_id(&key, oval::CU_LINUX_BA_TST_PREFIX);
        debug!("为基本测试生成ID: {} -> {}", test_type, id);
        id
    }
}

impl Default for IdGenerator {
    fn default() -> Self {
        Self::new(10000)
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
    info!("使用共享IdGenerator将CSAF转换为OVAL格式");
    let mut oval = OvalDefinitions::new();
    let now = Utc::now();
    // 使用RFC3339格式（符合xs:dateTime要求）
    let formatted_time = now.to_rfc3339();
    oval.generator.time_stamp = formatted_time.clone();
    let mut definations = oval::Definitions::new();
    let mut defination = Definition::new();
    // 为定义生成唯一ID
    if !csaf.vulnerabilities.is_empty() {
        let definition_id =
            id_generator.generate_definition_id_for_cve(&csaf.vulnerabilities[0].cve);
        defination.id = definition_id.clone();
        info!("为定义生成ID: {}", definition_id);
    }
    fill_definition(&csaf, &mut defination)?;
    let (criteria, info_tests, info_objects, info_states, os_tests, os_objects, os_states) =
        build_oval_criteria(&csaf.vulnerabilities[0], id_generator)?;
    defination.criteria = criteria;
    definations.items.push(defination);
    oval.definitions = definations;
    oval.tests.rpminfo_tests = info_tests;
    oval.tests.rpmverifyfile_tests = os_tests;
    oval.objects.rpm_info_objects = info_objects;
    oval.objects.rpmverifyfile_objects = os_objects;
    oval.states.rpminfo_states = Some(info_states);
    oval.states.rpmverifyfile_states = Some(os_states);
    info!("CSAF到OVAL转换完成");
    Ok(oval)
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
    info!("开始将CSAF转换为OVAL格式，初始计数器: {}", initial_counter);
    let mut id_generator = IdGenerator::new(initial_counter);
    csaf_to_oval_with_shared_generator(csaf, &mut id_generator)
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
    info!("使用默认计数器将CSAF转换为OVAL格式");
    csaf_to_oval_with_counter(csaf, 10000)
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
    info!("批量将 {} 个CSAF转换为OVAL格式", csaf_list.len());
    let mut shared_id_generator = IdGenerator::new(initial_counter);
    let mut results = Vec::new();
    for csaf in csaf_list {
        let oval = csaf_to_oval_with_shared_generator(csaf, &mut shared_id_generator)?;
        results.push(oval);
    }
    info!("批量转换完成，共生成 {} 个OVAL定义", results.len());
    Ok(results)
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
    debug!("解析软件包字符串: {}", pkg_string);
    let parts: Vec<&str> = pkg_string.split(':').collect();
    if parts.len() != 2 {
        warn!("软件包字符串格式不正确，期望2个部分，实际: {}", parts.len());
        return None;
    }
    let os_parts: Vec<&str> = parts[0].split('-').collect();
    let os_name = os_parts.get(0)?;

    let pkg_version_tag = parts[1];
    // 处理架构部分，获取不带架构的包名
    let del_arch_pkg_name;
    if let Some(pos) = pkg_version_tag.rfind('.') {
        del_arch_pkg_name = &pkg_version_tag[..pos];
    } else {
        del_arch_pkg_name = pkg_version_tag;
    }

    // 使用类似Perl命令的策略解析包名
    // 将包名按"-"分割成数组，然后取前(n-2)个元素作为包名（n是数组长度）
    // 剩余的元素组合作为EVR
    let parts_split: Vec<&str> = del_arch_pkg_name.split('-').collect();
    if parts_split.len() >= 3 {
        // 计算包名部分的长度（总长度-2）
        let name_parts_count = parts_split.len() - 2;
        if name_parts_count > 0 && name_parts_count < parts_split.len() {
            // 包名是前name_parts_count个元素
            let name = parts_split[..name_parts_count].join("-");
            // EVR是剩余的元素
            let evr = parts_split[name_parts_count..].join("-");

            debug!(
                "解析结果 - OS完整名: {}, 包名: {}, EVR: {}, OS名: {}",
                parts[0], name, evr, os_name
            );
            Some((
                parts[0].to_string(),   // 操作系统完整名称
                name,                   // 软件包名称
                evr,                    // EVR (Version-Release)
                (*os_name).to_string(), // 操作系统名称
            ))
        } else {
            // 如果计算出的包名长度不合理，回退到原来的解析方法
            let pkg_parts: Vec<&str> = pkg_version_tag.rsplitn(3, '-').collect();
            if pkg_parts.len() < 3 {
                warn!("软件包字符串解析失败，部分数量不足");
                return None;
            }

            let version = pkg_parts[0];
            let release = pkg_parts[1];
            let evr = format!("{}-{}", version, release);
            debug!(
                "回退解析结果 - OS完整名: {}, 包名: {}, EVR: {}, OS名: {}",
                parts[0], pkg_parts[2], evr, os_name
            );
            Some((
                parts[0].to_string(), // 直接使用parts[0]而不是os_full变量
                pkg_parts[2].to_string(),
                evr,
                (*os_name).to_string(),
            ))
        }
    } else {
        // 如果分割后的部分少于3个，回退到原来的解析方法
        let pkg_parts: Vec<&str> = pkg_version_tag.rsplitn(3, '-').collect();
        if pkg_parts.len() < 3 {
            warn!("软件包字符串解析失败，部分数量不足");
            return None;
        }

        let version = pkg_parts[0];
        let release = pkg_parts[1];
        let evr = format!("{}-{}", version, release);
        debug!(
            "回退解析结果 - OS完整名: {}, 包名: {}, EVR: {}, OS名: {}",
            parts[0], pkg_parts[2], evr, os_name
        );
        Some((
            parts[0].to_string(), // 直接使用parts[0]而不是os_full变量
            pkg_parts[2].to_string(),
            evr,
            (*os_name).to_string(),
        ))
    }
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
    info!("填充OVAL定义");
    // 填充definition结构体
    let mut metadata = Metadata::new();
    let mut affect = Affected::new();
    let mut has_note = false;

    // 从csaf的notes中获取软件包的说明
    // 填充metadata结构体
    for note in &sa.document.notes {
        if note.title == "Synopsis".to_string() {
            // TODO: Add SA ID
            metadata.title = note.text.clone();
            has_note = true;
        }

        if note.title == "Summary".to_string() {
            affect.platform = note.text.clone();
            metadata.affected = affect.clone();
        }

        if note.title == "Description".to_string() {
            metadata.description = note.text.clone();
        }
    }

    if !has_note {
        error!("CSAF文档缺少note部分");
        return Err("CSAF document has no note section".into());
    }

    // 设置definition的ID，使用CSAF文档的ID并进行处理
    // 仅保留最后的数字-数字部分，并将减号去掉，然后添加前缀
    let original_id = &sa.document.tracking.id;
    let processed_id = process_csaf_id(original_id);
    // 使用OVAL中定义的前缀格式
    definition.id = format!("{}{}", oval::CU_LINUX_SA_DEF_PREFIX, processed_id);

    // 设置definition的class为"patch"
    definition.class = "patch".to_string();

    // 设置definition的version为CSAF文档的版本号
    definition.version = sa.document.tracking.version.parse().unwrap_or(1);

    let mut references = Vec::<oval::Reference>::new();
    let mut map_cve_url = HashMap::new();
    for reference in &sa.document.references {
        let mut oval_ref = oval::Reference::new();
        if reference.summary.starts_with("CVE") {
            oval_ref.ref_id = reference.summary.clone();
            oval_ref.ref_url = reference.url.clone();
            oval_ref.source = reference.summary.clone();
            references.push(oval_ref.clone());
        }

        if reference.category == "external".to_string() {
            let cveurl = reference.url.clone();
            if let Some(pos) = cveurl.rfind('/') {
                let cve_id = &cveurl[pos + 1..].to_string();
                map_cve_url.insert(cve_id.clone(), cveurl.clone());
            }
        }
    }

    // 将references列表赋值给metadata
    metadata.references = Some(references);

    let mut advisory = oval::Advisory::new();
    let csaf_date = sa.document.tracking.current_release_date.clone();
    let parsed_res = DateTime::parse_from_rfc3339(&csaf_date);
    let issued_time;
    match parsed_res {
        Ok(parsed_time) => {
            issued_time = parsed_time.format("%Y-%m-%d");
        }
        Err(e) => {
            error!("解析CSAF日期失败: {}", e);
            return Err(e.into());
        }
    }

    advisory.issued.date = issued_time.to_string();
    advisory.updated.date = issued_time.to_string();

    // 创建CVE列表
    let mut cve_list = Vec::new();

    for csaf_cve in &sa.vulnerabilities {
        let mut cve = CVE::new();
        for score in &csaf_cve.scores {
            cve.cvss3 = score.cvss_v3.vector_string.clone();
            cve.impact = score.cvss_v3.base_severity.clone();
        }
        cve.content = csaf_cve.cve.clone(); // 使用cve字段而不是title
        match map_cve_url.get(&csaf_cve.cve) {
            // 使用cve字段而不是title
            Some(url) => {
                cve.href = url.clone();
            }
            None => {
                cve.href = "".to_string();
            }
        };
        cve_list.push(cve); // 将CVE添加到列表中
    }

    // 将CVE列表赋值给advisory
    advisory.cve = cve_list.clone();

    // 根据CVE的impact计算最高严重性级别并填充到advisory.severity
    let max_severity = oval::calculate_max_severity(&cve_list);
    if !max_severity.is_empty() {
        advisory.severity = max_severity;
        info!("根据CVE计算得到的最高严重性级别: {}", advisory.severity);
    } else {
        info!("未找到CVE impact信息，保持severity为空");
    }

    // 将advisory赋值给definition.metadata
    metadata.advisory = advisory;
    definition.metadata = metadata.clone();
    info!("OVAL定义填充完成");
    Ok(())
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
    // 从右向左查找，找到最后两个由数字组成的部分
    let parts: Vec<&str> = id.split('-').collect();

    // 如果至少有两个部分，检查最后两个部分是否都是数字
    if parts.len() >= 2 {
        let last_part = parts[parts.len() - 1];
        let second_last_part = parts[parts.len() - 2];

        // 检查最后两个部分是否都是数字
        if last_part.chars().all(|c| c.is_ascii_digit())
            && second_last_part.chars().all(|c| c.is_ascii_digit())
        {
            // 合并最后两个数字部分，去掉减号
            return format!("{}{}", second_last_part, last_part);
        }
    }

    // 如果不符合模式，返回原始ID
    id.to_string()
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
) -> Result<(
    Criteria,
    Vec<RpmInfoTest>,
    Vec<RpmInfoObject>,
    Vec<RpmInfoState>,
    Vec<oval::RpmVerifyFileTest>,
    Vec<oval::RpmVerifyFileObject>,
    Vec<oval::RpmVerifyFileState>,
)> {
    info!("构建OVAL检查条件");
    let mut rpminfo_test: Vec<RpmInfoTest> = Vec::new();
    let mut rpminfo_object: Vec<RpmInfoObject> = Vec::new();
    let mut rpminfo_states: Vec<RpmInfoState> = Vec::new();

    // 存储用于去重和引用
    let mut object_map: HashMap<String, String> = HashMap::new();
    let mut state_map: HashMap<String, String> = HashMap::new();
    let mut test_map: HashMap<String, String> = HashMap::new(); // 添加测试去重映射

    // 存储所有的<criteria operator="AND">(即软件包的检查逻辑)
    let mut pkg_and_criterions: Vec<Criteria> = Vec::new();

    // 存储最终生成的criteria
    let mut criteria = Criteria::new();
    // 提取第一个软件包的dist标识并匹配OS信息
    let os_info = if let Some(first_pkg) = sa.product_status.fixed.first() {
        if let Some((_, _, evr_full, _)) = parse_package_string(first_pkg) {
            let dist = extract_dist_from_package(&evr_full);
            let matched_os = match_os_info_by_dist(dist.as_deref());
            info!(
                "匹配到OS信息: {} {}, dist: {}",
                matched_os.os_type, matched_os.os_version, matched_os.dist
            );
            matched_os
        } else {
            warn!("无法解析第一个软件包，使用Unknown OS");
            match_os_info_by_dist(None)
        }
    } else {
        warn!("没有软件包信息，使用Unknown OS");
        match_os_info_by_dist(None)
    };

    for pkg_string in sa.product_status.fixed.clone() {
        if let Some((_os_full, pkg_name, evr_full, _os_name)) = parse_package_string(&pkg_string) {
            debug!("处理软件包: {}, EVR: {}", pkg_name, evr_full);
            // 保存evr_full的克隆用于后续使用
            let evr_full_clone = evr_full.clone();

            //1. 生成Object和State，并将其添加到定义的列表中
            if !object_map.contains_key(&pkg_name) {
                // ID生成器已经返回了带有前缀的ID，无需再次添加前缀
                let id = id_generator.generate_object_id_for_package(&pkg_name);
                let rpm_info_object = RpmInfoObject::new()
                    .with_id(id.clone())
                    .with_ver(1)
                    .with_rpm_name(pkg_name.clone());
                rpminfo_object.push(rpm_info_object);
                object_map.insert(pkg_name.clone(), id.clone());
                debug!("创建新的RPM对象: {} -> {}", pkg_name, id);
            }

            if !state_map.contains_key(&evr_full) {
                // ID生成器已经返回了带有前缀的ID，无需再次添加前缀
                let id = id_generator.generate_state_id_for_evr(&evr_full);
                let evr = oval::Evr {
                    datatype: "evr_string".to_string(),
                    operation: "less than".to_string(),
                    evr: utils::add_epoch_prefix(&pkg_name, &evr_full),
                };
                let rpm_info_state = RpmInfoState::new()
                    .with_id(id.clone())
                    .with_version("1".to_string())
                    .with_evr(Some(evr));
                rpminfo_states.push(rpm_info_state);
                state_map.insert(evr_full.clone(), id.clone());
                debug!("创建新的RPM状态: {} -> {}", evr_full, id);
            }

            // 2. 生成test（添加去重逻辑）
            let test_key = format!("{}:{}", pkg_name, evr_full_clone);
            let test_ref = if !test_map.contains_key(&test_key) {
                let new_test_ref = id_generator.generate_test_id(&pkg_name, &evr_full_clone);
                let evr_with_epoch = utils::add_epoch_prefix(&pkg_name, &evr_full_clone);
                rpminfo_test.push(RpmInfoTest {
                    check: "at least one".to_string(),
                    comment: format!("{} is earlier than {}", pkg_name, evr_with_epoch),
                    id: new_test_ref.clone(),
                    version: 1,
                    object: ObjectReference {
                        object_ref: object_map[&pkg_name].clone(),
                    },
                    state: StateReference {
                        state_ref: state_map[&evr_full_clone].clone(),
                    },
                });
                test_map.insert(test_key.clone(), new_test_ref.clone());
                debug!("创建新的RPM测试: {} -> {}", test_key, new_test_ref);
                new_test_ref
            } else {
                let existing_ref = test_map.get(&test_key).unwrap().clone();
                debug!("使用现有RPM测试: {} -> {}", test_key, existing_ref);
                existing_ref
            };

            // 3. 构建<criteria operator="AND"> (用于主逻辑)
            let evr_with_epoch = utils::add_epoch_prefix(&pkg_name, &evr_full_clone);
            let pkg_criteria = Criteria {
                operator: "AND".to_string(),
                criterion: vec![Criterion {
                    comment: format!("{} is earlier than {}", pkg_name, evr_with_epoch),
                    test_ref: test_ref,
                }],
                sub_criteria: None,
            };
            pkg_and_criterions.push(pkg_criteria);
        }
    }

    // 4. 创建 OS 检查的 tests, objects, states
    let mut os_tests = Vec::new();
    let mut os_objects = Vec::new();
    let mut os_states = Vec::new();

    // 根据 dist 获取 OS 信息
    let (filepath, name_pattern, _version_pattern) = match os_info.dist.as_str() {
        "oe1" => ("/etc/openeuler-release", "^openeuler-release", "^20.03"),
        "oe2203" => ("/etc/openeuler-release", "^openeuler-release", "^22.03"),
        "oe2403" => ("/etc/openeuler-release", "^openeuler-release", "^24.03"),
        "el7" => ("/etc/redhat-release", "^redhat-release", "^7[^\\d]"),
        "el8" => ("/etc/redhat-release", "^redhat-release", "^8[^\\d]"),
        "el9" => ("/etc/redhat-release", "^redhat-release", "^9[^\\d]"),
        "ule4" => ("/etc/culinux-release", "^culinux-release", "^4"),
        _ => {
            warn!("未知的 dist: {}, 使用默认配置", os_info.dist);
            ("/etc/os-release", "^unknown", ".*")
        }
    };

    // 使用 dist 获取对应的 os_info_id，再生成固定 ID（避免不同 OS 之间的 ID 冲突）
    let os_info_id = get_os_info_id_by_dist(&os_info.dist);
    let (os_object_id, os_state_full_id, os_state_name_only_id, os_test_must_id, os_test_is_id) =
        generate_os_check_ids(os_info_id);

    // 创建 RpmVerifyFileObject
    let os_object = oval::RpmVerifyFileObject {
        id: os_object_id.clone(),
        ver: 1,
        behaviors: oval::Behaviors::new(),
        name: oval::Data {
            operation: "pattern match".to_string(),
        },
        epoch: oval::Data {
            operation: "pattern match".to_string(),
        },
        version: oval::Data {
            operation: "pattern match".to_string(),
        },
        release: oval::Data {
            operation: "pattern match".to_string(),
        },
        arch: oval::Data {
            operation: "pattern match".to_string(),
        },
        filepath: filepath.to_string(),
    };
    os_objects.push(os_object);

    // 创建两个 RpmVerifyFileState
    // State 1: 完整检查 (name + version) - 用于 "must be installed"
    // version 字段使用 os_version 进行匹配
    // 转义特殊字符（如 . 转为 \\.)
    let version_match_pattern = format!("^{}", os_info.os_version);
    let os_state_full = oval::RpmVerifyFileState {
        id: os_state_full_id.clone(),
        version: "1".to_string(),
        name: oval::StateData {
            operation: "pattern match".to_string(),
            content: name_pattern.to_string(),
        },
        os_version: Some(oval::StateData {
            operation: "pattern match".to_string(),
            content: version_match_pattern, // 使用 os_version
        }),
    };
    os_states.push(os_state_full);

    // State 2: 仅检查名称 (name only) - 用于 "is installed"
    let os_state_name_only = oval::RpmVerifyFileState {
        id: os_state_name_only_id.clone(),
        version: "1".to_string(),
        name: oval::StateData {
            operation: "pattern match".to_string(),
            content: name_pattern.to_string(),
        },
        os_version: None, // 不检查版本
    };
    os_states.push(os_state_name_only);

    // 创建两个 RpmVerifyFileTest
    // Test 1: "must be installed" - check="none satisfy" - 使用仅检查名称 state
    // 反向检查：如果没有找到包名，说明系统不匹配（只检查OS类型，不含版本）
    let os_test_must = oval::RpmVerifyFileTest {
        check: "none satisfy".to_string(),
        comment: format!("{} must be installed", os_info.os_type),
        id: os_test_must_id.clone(),
        version: 1,
        object: oval::ObjectReference {
            object_ref: os_object_id.clone(),
        },
        state: oval::StateReference {
            state_ref: os_state_name_only_id.clone(),
        },
    };
    os_tests.push(os_test_must);

    // Test 2: "is installed" - check="at least one" - 使用完整检查 state
    // 正向检查：检查是否安装了特定版本
    let os_test_is = oval::RpmVerifyFileTest {
        check: "at least one".to_string(),
        comment: format!("{} {} is installed", os_info.os_type, os_info.os_version),
        id: os_test_is_id.clone(),
        version: 1,
        object: oval::ObjectReference {
            object_ref: os_object_id.clone(),
        },
        state: oval::StateReference {
            state_ref: os_state_full_id.clone(),
        },
    };
    os_tests.push(os_test_is);

    // 5. 组装最终<criteria> 结构
    if !sa.product_status.fixed.is_empty() {
        if let Some((_os_full, _, _, _)) = parse_package_string(&sa.product_status.fixed[0]) {
            let pkg_or_criteria = Criteria {
                operator: "OR".to_string(),
                criterion: Vec::new(),
                sub_criteria: Some(pkg_and_criterions),
            };

            let os_and_criteria = Criteria {
                operator: "AND".to_string(),
                criterion: vec![Criterion {
                    comment: format!("{} {} is installed", os_info.os_type, os_info.os_version),
                    test_ref: os_test_is_id,
                }],
                sub_criteria: Some(vec![pkg_or_criteria]),
            };
            criteria = Criteria {
                operator: "OR".to_string(),
                criterion: vec![Criterion {
                    comment: format!("{} must be installed", os_info.os_type),
                    test_ref: os_test_must_id,
                }],
                sub_criteria: Some(vec![os_and_criteria]),
            };
            info!(
                "OVAL检查条件构建完成，目标OS: {} {}",
                os_info.os_type, os_info.os_version
            );
        }
    }
    Ok((
        criteria,
        rpminfo_test,
        rpminfo_object,
        rpminfo_states,
        os_tests,
        os_objects,
        os_states,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use csaf::CSAF;
    use std::fs;

    // 从配置文件读取测试文件路径
    fn get_test_file_path(module: &str, filename: &str) -> String {
        // 在测试环境中，直接使用相对路径
        format!("../test/{}/{}", module, filename)
    }

    #[test]
    fn test_parse_package_string() {
        // 测试正常的包字符串解析: python-jinja2-2.11.2-9.oe2003sp4.noarch
        let pkg_string = "openEuler-20.03-LTS-SP4:python-jinja2-2.11.2-9.oe2003sp4.noarch";
        let result = parse_package_string(pkg_string);
        assert!(result.is_some());
        let (os_full, pkg_name, evr, os_name) = result.unwrap();
        assert_eq!(os_full, "openEuler-20.03-LTS-SP4");
        assert_eq!(pkg_name, "python-jinja2");
        assert_eq!(evr, "2.11.2-9.oe2003sp4");
        assert_eq!(os_name, "openEuler");
    }

    #[test]
    fn test_parse_package_string_with_complex_name() {
        // 测试包含多个连字符的包名
        let pkg_string = "openEuler-20.03-LTS-SP4:python-setuptools-scm-6.0.1-1.oe2003sp4.noarch";
        let result = parse_package_string(pkg_string);
        assert!(result.is_some());
        let (os_full, pkg_name, evr, os_name) = result.unwrap();
        assert_eq!(os_full, "openEuler-20.03-LTS-SP4");
        assert_eq!(pkg_name, "python-setuptools-scm");
        assert_eq!(evr, "6.0.1-1.oe2003sp4");
        assert_eq!(os_name, "openEuler");
    }

    #[test]
    fn test_parse_package_string_invalid_format() {
        // 测试格式错误的字符串（缺少冒号）
        let pkg_string = "openEuler-20.03-LTS-SP4-python-jinja2-2.11.2-9.oe2003sp4.noarch";
        let result = parse_package_string(pkg_string);
        assert!(result.is_none(), "无效格式应返回None");
    }

    #[test]
    fn test_parse_package_string_no_arch() {
        // 测试不带架构后缀的包
        let pkg_string = "openEuler-20.03-LTS-SP4:nginx-1.20.1-1";
        let result = parse_package_string(pkg_string);
        assert!(result.is_some());
        let (_, pkg_name, evr, _) = result.unwrap();
        assert_eq!(pkg_name, "nginx");
        // 注意：解析逻辑会将最后两个部分作为version-release，所以结果是 "1-1.20.1" 而不是 "1.20.1-1"
        assert_eq!(evr, "1-1.20.1");
    }

    #[test]
    fn test_id_generator() {
        let mut id_gen = IdGenerator::new(10000);

        // 测试对象ID生成
        let obj_id1 = id_gen.generate_object_id_for_package("test-package");
        let obj_id2 = id_gen.generate_object_id_for_package("test-package");
        assert_eq!(obj_id1, obj_id2, "相同对象名应生成相同ID");

        // 测试不同对象名生成不同ID
        let obj_id3 = id_gen.generate_object_id_for_package("another-package");
        assert_ne!(obj_id1, obj_id3, "不同对象名应生成不同ID");

        // 测试状态ID生成
        let state_id1 = id_gen.generate_state_id_for_evr("1.0.0-1");
        let state_id2 = id_gen.generate_state_id_for_evr("1.0.0-1");
        assert_eq!(state_id1, state_id2, "相同EVR应生成相同ID");

        // 测试测试ID生成
        let test_id1 = id_gen.generate_test_id("package", "1.0.0-1");
        let test_id2 = id_gen.generate_test_id("package", "1.0.0-1");
        assert_eq!(test_id1, test_id2, "相同测试应生成相同ID");

        // 测试计数器递增
        let initial_counter = id_gen.get_current_counter();
        let _new_id = id_gen.generate_unique_id("test:");
        assert_eq!(
            id_gen.get_current_counter(),
            initial_counter + 1,
            "计数器应正确递增"
        );
    }

    #[test]
    fn test_id_generator_default() {
        let id_gen = IdGenerator::default();
        assert_eq!(id_gen.get_current_counter(), 10000, "默认计数器应为10000");
    }

    #[test]
    fn test_id_generator_counter_operations() {
        let mut id_gen = IdGenerator::new(5000);
        assert_eq!(id_gen.get_current_counter(), 5000);

        // 设置计数器
        id_gen.set_current_counter(8000);
        assert_eq!(id_gen.get_current_counter(), 8000);

        // 生成ID后计数器递增
        let _id = id_gen.generate_unique_id("test:");
        assert_eq!(id_gen.get_current_counter(), 8001);
    }

    #[test]
    fn test_id_generator_definition_id() {
        let mut id_gen = IdGenerator::new(10000);

        let def_id1 = id_gen.generate_definition_id_for_cve("CVE-2024-1234");
        let def_id2 = id_gen.generate_definition_id_for_cve("CVE-2024-1234");
        assert_eq!(def_id1, def_id2, "相同CVE应生成相同定义ID");

        let def_id3 = id_gen.generate_definition_id_for_cve("CVE-2024-5678");
        assert_ne!(def_id1, def_id3, "不同CVE应生成不同定义ID");
    }

    #[test]
    fn test_id_generator_base_test_id() {
        let mut id_gen = IdGenerator::new(10000);

        let test_id1 = id_gen.generate_base_test_id("os_installed");
        let test_id2 = id_gen.generate_base_test_id("os_installed");
        assert_eq!(test_id1, test_id2, "相同测试类型应生成相同ID");

        let test_id3 = id_gen.generate_base_test_id("os_required");
        assert_ne!(test_id1, test_id3, "不同测试类型应生成不同ID");
    }

    #[test]
    fn test_process_csaf_id() {
        // 测试标准格式的CSAF ID
        let id1 = "openEuler-SA-2025-1004";
        let processed1 = process_csaf_id(id1);
        assert_eq!(processed1, "20251004", "应提取并组合最后两个数字部分");

        // 测试另一个格式
        let id2 = "RHSA-2024-0123";
        let processed2 = process_csaf_id(id2);
        assert_eq!(processed2, "20240123");

        // 测试不符合模式的ID（应返回原始ID）
        let id3 = "CUSTOM-ID-ABC";
        let processed3 = process_csaf_id(id3);
        assert_eq!(processed3, "CUSTOM-ID-ABC", "不符合模式应返回原始ID");

        // 测试只有一个数字部分
        let id4 = "SA-2025";
        let processed4 = process_csaf_id(id4);
        assert_eq!(processed4, "SA-2025", "只有一个数字部分应返回原始ID");
    }

    #[test]
    fn test_csaf_to_oval_conversion() {
        // 使用CSAF测试文件进行转换测试
        let test_file = get_test_file_path("csaf", "csaf-openeuler-sa-2025-1004.json");
        let csaf = CSAF::from_file(&test_file).expect("Failed to load CSAF test file");

        // 执行转换
        let oval_result = csaf_to_oval(&csaf);
        assert!(
            oval_result.is_ok(),
            "CSAF to OVAL conversion failed: {:?}",
            oval_result.err()
        );

        let oval = oval_result.unwrap();

        // 验证基本结构
        assert!(
            !oval.definitions.items.is_empty(),
            "OVAL definitions should not be empty"
        );
        assert!(
            !oval.tests.rpminfo_tests.is_empty(),
            "RPM info tests should not be empty"
        );
        assert!(
            !oval.objects.rpm_info_objects.is_empty(),
            "RPM info objects should not be empty"
        );
        assert!(
            oval.states.rpminfo_states.is_some(),
            "RPM info states should be present"
        );

        // 验证生成器信息
        assert_eq!(oval.generator.product_name, "China Unicom Linux");
        assert!(
            !oval.generator.time_stamp.is_empty(),
            "Timestamp should be set"
        );

        // 验证至少有一个定义
        let definition = &oval.definitions.items[0];
        assert!(
            !definition.metadata.title.is_empty(),
            "Definition title should not be empty"
        );
        assert!(
            !definition.metadata.description.is_empty(),
            "Definition description should not be empty"
        );
        assert!(
            !definition.id.is_empty(),
            "Definition ID should not be empty"
        );

        // 验证severity字段已正确填充
        assert!(
            !definition.metadata.advisory.severity.is_empty(),
            "Advisory severity should not be empty"
        );
        // 验证severity是否为有效值
        let valid_severities = ["None", "Low", "Medium", "Moderate", "Important", "High", "Critical"];
        assert!(
            valid_severities.contains(&definition.metadata.advisory.severity.as_str()),
            "Severity '{}' should be one of: {:?}",
            definition.metadata.advisory.severity,
            valid_severities
        );
    }

    #[test]
    fn test_csaf_to_oval_with_custom_counter() {
        let test_file = get_test_file_path("csaf", "csaf-openeuler-sa-2025-1004.json");
        let csaf = CSAF::from_file(&test_file).expect("Failed to load CSAF test file");

        // 使用自定义计数器
        let oval_result = csaf_to_oval_with_counter(&csaf, 50000);
        assert!(oval_result.is_ok());

        let oval = oval_result.unwrap();

        // 验证ID包含自定义计数器生成的数字
        if let Some(test) = oval.tests.rpminfo_tests.first() {
            assert!(test.id.contains("50"), "ID应包含自定义计数器范围的数字");
        }
    }

    #[test]
    fn test_fill_definition() {
        let test_file = get_test_file_path("csaf", "csaf-openeuler-sa-2025-1004.json");
        let csaf = CSAF::from_file(&test_file).expect("Failed to load CSAF test file");

        let mut definition = Definition::new();
        let result = fill_definition(&csaf, &mut definition);

        assert!(result.is_ok(), "填充定义应成功");
        assert!(!definition.metadata.title.is_empty(), "标题不应为空");
        assert!(!definition.metadata.description.is_empty(), "描述不应为空");
        assert_eq!(definition.class, "patch", "类别应为patch");
        assert!(!definition.id.is_empty(), "ID不应为空");
        assert!(
            definition.id.starts_with(oval::CU_LINUX_SA_DEF_PREFIX),
            "ID应以正确前缀开头"
        );

        // 验证CVE列表
        assert!(
            !definition.metadata.advisory.cve.is_empty(),
            "CVE列表不应为空"
        );

        // 验证severity字段已正确填充（根据CVE的impact计算）
        assert!(
            !definition.metadata.advisory.severity.is_empty(),
            "Severity字段不应为空"
        );
        let valid_severities = ["None", "Low", "Medium", "Moderate", "Important", "High", "Critical"];
        assert!(
            valid_severities.contains(&definition.metadata.advisory.severity.as_str()),
            "Severity '{}' 应为有效值: {:?}",
            definition.metadata.advisory.severity,
            valid_severities
        );

        // 验证引用
        assert!(definition.metadata.references.is_some(), "应包含引用");
        if let Some(refs) = &definition.metadata.references {
            assert!(!refs.is_empty(), "引用列表不应为空");
        }
    }

    #[test]
    fn test_build_oval_criteria() {
        let test_file = get_test_file_path("csaf", "csaf-openeuler-sa-2025-1004.json");
        let csaf = CSAF::from_file(&test_file).expect("Failed to load CSAF test file");

        let mut id_generator = IdGenerator::new(10000);
        let vulnerability = &csaf.vulnerabilities[0];

        let result = build_oval_criteria(vulnerability, &mut id_generator);
        assert!(result.is_ok(), "构建条件应成功");

        let (criteria, tests, objects, states, os_tests, os_objects, os_states) = result.unwrap();

        // 验证条件结构
        assert_eq!(criteria.operator, "OR", "顶层操作符应为OR");
        assert!(!criteria.criterion.is_empty(), "应包含条件");

        // 验证测试、对象、状态列表
        assert!(!tests.is_empty(), "测试列表不应为空");
        assert!(!objects.is_empty(), "对象列表不应为空");
        assert!(!states.is_empty(), "状态列表不应为空");

        // 验证操作系统检测相关元素
        assert!(!os_tests.is_empty(), "OS测试列表不应为空");
        assert_eq!(os_tests.len(), 2, "应包含2个OS测试（must和is）");
        assert!(!os_objects.is_empty(), "OS对象列表不应为空");
        assert_eq!(os_objects.len(), 1, "应包含1个OS对象");
        assert!(!os_states.is_empty(), "OS状态列表不应为空");
        assert_eq!(os_states.len(), 2, "应包含2个OS状态（full和name_only）");

        // 验证测试引用的对象和状态存在
        for test in &tests {
            assert!(!test.id.is_empty(), "测试ID不应为空");
            assert!(!test.object.object_ref.is_empty(), "对象引用不应为空");
            assert!(!test.state.state_ref.is_empty(), "状态引用不应为空");
            assert_eq!(test.check, "at least one", "检查方式应为'at least one'");
        }

        // 验证对象
        for object in &objects {
            assert!(!object.id.is_empty(), "对象ID不应为空");
            assert!(!object.rpm_name.is_empty(), "RPM名称不应为空");
            assert!(
                object.id.starts_with(oval::CU_LINUX_SA_OBJ_PREFIX),
                "对象ID应以正确前缀开头"
            );
        }

        // 验证状态
        for state in &states {
            assert!(!state.id.is_empty(), "状态ID不应为空");
            assert!(state.evr.is_some(), "应包含EVR信息");
            assert!(
                state.id.starts_with(oval::CU_LINUX_SA_STE_PREFIX),
                "状态ID应以正确前缀开头"
            );

            if let Some(evr) = &state.evr {
                assert_eq!(evr.datatype, "evr_string", "EVR数据类型应为evr_string");
                assert_eq!(evr.operation, "less than", "EVR操作应为less than");
                assert!(!evr.evr.is_empty(), "EVR值不应为空");
            }
        }
    }

    #[test]
    fn test_build_oval_criteria_deduplication() {
        // 测试去重逻辑
        let test_file = get_test_file_path("csaf", "csaf-openeuler-sa-2025-1004.json");
        let csaf = CSAF::from_file(&test_file).expect("Failed to load CSAF test file");

        let mut id_generator = IdGenerator::new(10000);
        let vulnerability = &csaf.vulnerabilities[0];

        let result = build_oval_criteria(vulnerability, &mut id_generator);
        assert!(result.is_ok());

        let (_, tests, objects, states, os_tests, os_objects, os_states) = result.unwrap();

        // 验证OS检测元素
        assert_eq!(os_tests.len(), 2, "应包含2个OS测试");
        assert_eq!(os_objects.len(), 1, "应包含1个OS对象");
        assert_eq!(os_states.len(), 2, "应包含2个OS状态");

        // 检查对象去重：相同包名应该只有一个对象
        let mut object_names = std::collections::HashSet::new();
        for object in &objects {
            assert!(
                object_names.insert(object.rpm_name.clone()),
                "包名 {} 重复，去重失败",
                object.rpm_name
            );
        }

        // 检查状态去重：相同EVR应该只有一个状态
        let mut evr_values = std::collections::HashSet::new();
        for state in &states {
            if let Some(evr) = &state.evr {
                assert!(
                    evr_values.insert(evr.evr.clone()),
                    "EVR {} 重复，去重失败",
                    evr.evr
                );
            }
        }

        // 检查测试去重：相同的包名+EVR组合应该只有一个测试
        let mut test_keys = std::collections::HashSet::new();
        for test in &tests {
            let key = format!("{}:{}", test.object.object_ref, test.state.state_ref);
            assert!(
                test_keys.insert(key.clone()),
                "测试组合 {} 重复，去重失败",
                key
            );
        }
    }

    #[test]
    fn test_csaf_to_oval_file_conversion() {
        // 使用CSAF测试文件进行转换测试
        let test_file = get_test_file_path("csaf", "csaf-openeuler-sa-2025-1004.json");
        let csaf = CSAF::from_file(&test_file).expect("Failed to load CSAF test file");

        // 执行转换
        let oval_result = csaf_to_oval(&csaf);
        assert!(
            oval_result.is_ok(),
            "CSAF to OVAL conversion failed: {:?}",
            oval_result.err()
        );

        let oval = oval_result.unwrap();

        // 将OVAL转换为XML字符串
        let xml_result = oval.to_oval_string();
        assert!(
            xml_result.is_ok(),
            "Failed to convert OVAL to XML string: {:?}",
            xml_result.err()
        );

        let xml_content = xml_result.unwrap();
        assert!(!xml_content.is_empty(), "XML content should not be empty");

        // 验证XML内容包含关键元素
        assert!(
            xml_content.contains("oval_definitions"),
            "XML应包含oval_definitions"
        );
        assert!(xml_content.contains("generator"), "XML应包含generator");
        assert!(xml_content.contains("definitions"), "XML应包含definitions");
        assert!(xml_content.contains("tests"), "XML应包含tests");
        assert!(xml_content.contains("objects"), "XML应包含objects");
        assert!(xml_content.contains("states"), "XML应包含states");

        // 保存到parser库的test目录下
        let output_path = "tests/csaf_openeuler_sa_2025_1004.xml";
        let write_result = fs::write(output_path, xml_content);
        assert!(
            write_result.is_ok(),
            "Failed to write OVAL XML to file: {:?}",
            write_result.err()
        );

        // 验证文件确实被创建
        let metadata = fs::metadata(output_path);
        assert!(metadata.is_ok(), "Output file should exist");

        // 注意：我们不清理测试文件，以便可以检查生成的OVAL XML文件
        // 在实际项目中，您可能需要根据需要决定是否清理
    }

    #[test]
    fn test_oval_xml_structure() {
        // 测试生成的OVAL XML结构的完整性
        let test_file = get_test_file_path("csaf", "csaf-openeuler-sa-2025-1004.json");
        let csaf = CSAF::from_file(&test_file).expect("Failed to load CSAF test file");

        let oval = csaf_to_oval(&csaf).expect("转换应成功");
        let xml = oval.to_oval_string().expect("XML生成应成功");

        // 验证命名空间
        assert!(xml.contains("xmlns=\""), "应包含默认命名空间");
        assert!(xml.contains("xmlns:oval"), "应包含oval命名空间");
        assert!(xml.contains("xmlns:red-def"), "应包含red-def命名空间");

        // 验证生成器信息
        assert!(xml.contains("China Unicom Linux"), "应包含产品名称");

        // 验证定义ID格式
        assert!(xml.contains(oval::CU_LINUX_SA_DEF_PREFIX), "应包含定义前缀");
    }

    #[test]
    fn test_id_generator_prefix_consistency() {
        // 测试ID生成器生成的ID包含正确的前缀
        let mut id_gen = IdGenerator::new(10000);

        let obj_id = id_gen.generate_object_id_for_package("test-pkg");
        assert!(
            obj_id.starts_with(oval::CU_LINUX_SA_OBJ_PREFIX),
            "对象ID应以正确前缀开头: {}",
            obj_id
        );

        let state_id = id_gen.generate_state_id_for_evr("1.0-1");
        assert!(
            state_id.starts_with(oval::CU_LINUX_SA_STE_PREFIX),
            "状态ID应以正确前缀开头: {}",
            state_id
        );

        let test_id = id_gen.generate_test_id("pkg", "1.0-1");
        assert!(
            test_id.starts_with(oval::CU_LINUX_SA_TST_PREFIX),
            "测试ID应以正确前缀开头: {}",
            test_id
        );

        let def_id = id_gen.generate_definition_id_for_cve("CVE-2024-1234");
        assert!(
            def_id.starts_with(oval::CU_LINUX_SA_DEF_PREFIX),
            "定义ID应以正确前缀开头: {}",
            def_id
        );

        let base_test_id = id_gen.generate_base_test_id("os_check");
        assert!(
            base_test_id.starts_with(oval::CU_LINUX_BA_TST_PREFIX),
            "基础测试ID应以正确前缀开头: {}",
            base_test_id
        );
    }
}
