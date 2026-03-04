//! 基于数据库的CSAF到OVAL转换器
//!
//! 该模块提供了使用数据库持久化ID计数器的CSAF到OVAL转换功能。

use crate::{DatabaseIdGenerator, DatabaseManager};
use chrono::{DateTime, Utc};
use csaf::{CSAF, Vulnerabilitie};
use log::{debug, error, info, warn};
use oval::{
    Affected, CVE, Criteria, Criterion, Definition, Metadata, ObjectReference, OvalDefinitions,
    RpmInfoObject, RpmInfoState, RpmInfoTest, StateReference,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use utils::Result;

/// 将CSAF格式转换为OVAL格式（使用数据库持久化ID计数器）
///
/// # 参数
///
/// * `csaf` - CSAF结构体的引用
/// * `db_manager` - 数据库管理器的引用
/// * `counter_id` - ID计数器ID
/// * `initial_counter` - 初始计数器值，用于确保ID唯一性
///
/// # 返回值
///
/// 返回Result<OvalDefinitions>，成功时包含转换后的OVAL定义，失败时包含错误信息
pub async fn csaf_to_oval_with_db_counter(
    csaf: &CSAF,
    db_manager: Arc<Mutex<DatabaseManager>>,
    counter_id: String,
    initial_counter: u64,
) -> Result<OvalDefinitions> {
    info!(
        "开始将CSAF转换为OVAL格式，使用数据库计数器: {}, 初始计数器: {}",
        counter_id, initial_counter
    );

    let mut oval = OvalDefinitions::new();
    let now = Utc::now();
    // 使用RFC3339格式（符合xs:dateTime要求）
    let formatted_time = now.to_rfc3339();
    oval.generator.time_stamp = formatted_time.clone();
    // TODO: Set the content_version rule
    // oval.generator.content_version = 0;

    let mut id_generator = DatabaseIdGenerator::new(db_manager, counter_id, initial_counter);

    let mut definations = oval::Definitions::new();
    let mut defination = Definition::new();

    // 为定义生成唯一ID
    if !csaf.vulnerabilities.is_empty() {
        let definition_id = id_generator
            .generate_definition_id_for_cve(&csaf.vulnerabilities[0].cve)
            .await?;
        defination.id = definition_id.clone();
        info!("为定义生成ID: {}", definition_id);
    }

    fill_definition(&csaf, &mut defination)?;
    let (criteria, info_tests, info_objects, info_states) =
        build_oval_criteria(&csaf.vulnerabilities[0], &mut id_generator).await?;
    defination.criteria = criteria;
    definations.items.push(defination);
    oval.definitions = definations; // 修复：将定义列表赋值给OVAL对象
    oval.tests.rpminfo_tests = info_tests;
    oval.objects.rpm_info_objects = info_objects;
    oval.states.rpminfo_states = Some(info_states);

    info!("CSAF到OVAL转换完成");
    Ok(oval)
}

/// 将CSAF格式转换为OVAL格式（使用默认数据库计数器）
///
/// # 参数
///
/// * `csaf` - CSAF结构体的引用
/// * `db_manager` - 数据库管理器的引用
///
/// # 返回值
///
/// 返回Result<OvalDefinitions>，成功时包含转换后的OVAL定义，失败时包含错误信息
pub async fn csaf_to_oval_with_default_db_counter(
    csaf: &CSAF,
    db_manager: Arc<Mutex<DatabaseManager>>,
) -> Result<OvalDefinitions> {
    info!("使用默认数据库计数器将CSAF转换为OVAL格式");
    csaf_to_oval_with_db_counter(csaf, db_manager, "default_counter".to_string(), 10000).await
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
fn fill_definition(sa: &CSAF, definition: &mut Definition) -> Result<()> {
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
fn process_csaf_id(id: &str) -> String {
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
/// 返回Result<(Criteria, Vec<RpmInfoTest>, Vec<RpmInfoObject>, Vec<RpmInfoState>)>，
/// 包含构建的检查条件和相关的测试、对象、状态列表
async fn build_oval_criteria(
    sa: &Vulnerabilitie,
    id_generator: &mut DatabaseIdGenerator,
) -> Result<(
    Criteria,
    Vec<RpmInfoTest>,
    Vec<RpmInfoObject>,
    Vec<RpmInfoState>,
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

    for pkg_string in sa.product_status.fixed.clone() {
        if let Some((_os_full, pkg_name, evr_full, _os_name)) = parse_package_string(&pkg_string) {
            debug!("处理软件包: {}, EVR: {}", pkg_name, evr_full);
            // 保存evr_full的克隆用于后续使用
            let evr_full_clone = evr_full.clone();

            //1. 生成Object和State，并将其添加到定义的列表中
            if !object_map.contains_key(&pkg_name) {
                // ID生成器已经返回了带有前缀的ID，无需再次添加前缀
                let id = id_generator
                    .generate_object_id_for_package(&pkg_name)
                    .await?;
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
                let id = id_generator.generate_state_id_for_evr(&evr_full).await?;
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
                let new_test_ref = id_generator
                    .generate_test_id(&pkg_name, &evr_full_clone)
                    .await?;
                rpminfo_test.push(RpmInfoTest {
                    check: "at least one".to_string(),
                    comment: format!("{} is earlier than {}", pkg_name, evr_full_clone),
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
            let pkg_criteria = Criteria {
                operator: "AND".to_string(),
                criterion: vec![Criterion {
                    comment: format!("{} is earlier than {}", pkg_name, evr_full_clone),
                    test_ref: test_ref,
                }],
                sub_criteria: None,
            };
            pkg_and_criterions.push(pkg_criteria);
        }
    }

    // 4. 组装最终<criteria> 结构
    if !sa.product_status.fixed.is_empty() {
        if let Some((os_full, _, _, _)) = parse_package_string(&sa.product_status.fixed[0]) {
            let pkg_or_criteria = Criteria {
                operator: "OR".to_string(),
                criterion: Vec::new(),
                sub_criteria: Some(pkg_and_criterions),
            };

            let os_and_criteria = Criteria {
                operator: "AND".to_string(),
                criterion: vec![Criterion {
                    comment: format!("{} is installed", os_full),
                    test_ref: id_generator.generate_base_test_id("os_installed").await?,
                }],
                sub_criteria: Some(vec![pkg_or_criteria]),
            };

            criteria = Criteria {
                operator: "OR".to_string(),
                criterion: vec![Criterion {
                    comment: format!("{} must be installed", os_full),
                    test_ref: id_generator.generate_base_test_id("os_required").await?,
                }],
                sub_criteria: Some(vec![os_and_criteria]),
            };
            info!("OVAL检查条件构建完成");
        }
    }
    Ok((criteria, rpminfo_test, rpminfo_object, rpminfo_states))
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
fn parse_package_string(pkg_string: &str) -> Option<(String, String, String, String)> {
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
