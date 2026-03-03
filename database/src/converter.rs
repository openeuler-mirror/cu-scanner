//! OVAL结构体到数据库结构体的转换函数
//!
//! 该模块提供了将OVAL定义转换为数据库实体的功能。

use crate::{
    Criteria, Criterion, Cve, OvalDefinition, Reference, RpmInfoObject, RpmInfoState, RpmInfoTest,
};

// 注意：这里需要从OVAL模块导入结构体
// 由于我们是在不同的crate中，需要在Cargo.toml中添加依赖
// 在实际项目中，请确保正确设置了依赖关系

/// 将OVAL Definitions转换为数据库OvalDefinition
///
/// # 参数
///
/// * `definition` - OVAL定义
///
/// # 返回值
///
/// 返回转换后的数据库OvalDefinition
pub fn convert_oval_definition_to_db(definition: &oval::Definition) -> OvalDefinition {
    OvalDefinition {
        id: definition.id.clone(),
        class: definition.class.clone(),
        version: definition.version,
        title: definition.metadata.title.clone(),
        description: definition.metadata.description.clone(),
        family: definition.metadata.affected.family.clone(),
        platform: definition.metadata.affected.platform.clone(),
        severity: definition.metadata.advisory.severity.clone(),
        rights: definition.metadata.advisory.rights.clone(),
        from: definition.metadata.advisory.from.clone(),
        issued_date: definition.metadata.advisory.issued.date.clone(),
        updated_date: definition.metadata.advisory.updated.date.clone(),
        os_info_id: None, // 将在存储时根据软件包版本匹配
    }
}

/// 转换完整的OVAL定义（包括所有子项目）
///
/// # 参数
///
/// * `definition` - OVAL定义
/// * `tests` - 测试信息
/// * `objects` - 对象信息
/// * `states` - 状态信息
///
/// # 返回值
///
/// 返回转换后的数据库实体元组 (OvalDefinition, Vec<Reference>, Vec<Cve>, Vec<RpmInfoTest>, Vec<RpmInfoObject>, Vec<RpmInfoState>)
pub fn convert_full_oval_definition(
    definition: &oval::Definition,
    tests: &oval::Tests,
    objects: &oval::Objects,
    states: &oval::States,
) -> (
    OvalDefinition,
    Vec<Reference>,
    Vec<Cve>,
    Vec<RpmInfoTest>,
    Vec<RpmInfoObject>,
    Vec<RpmInfoState>,
) {
    (
        convert_oval_definition_to_db(definition),
        convert_references(&definition.metadata.references),
        convert_cves(&definition.metadata.advisory.cve),
        convert_rpminfo_tests(tests),
        convert_rpminfo_objects(objects),
        convert_rpminfo_states(states),
    )
}

/// 转换引用信息
///
/// # 参数
///
/// * `references` - OVAL引用信息
///
/// # 返回值
///
/// 返回转换后的数据库Reference列表
pub fn convert_references(references: &Option<Vec<oval::Reference>>) -> Vec<Reference> {
    match references {
        Some(refs) => refs
            .iter()
            .map(|r| Reference {
                ref_id: r.ref_id.clone(),
                ref_url: r.ref_url.clone(),
                source: r.source.clone(),
            })
            .collect(),
        None => Vec::new(),
    }
}

/// 转换CVE信息
///
/// # 参数
///
/// * `cves` - OVAL CVE信息
///
/// # 返回值
///
/// 返回转换后的数据库Cve列表
pub fn convert_cves(cves: &Vec<oval::CVE>) -> Vec<Cve> {
    cves.iter()
        .map(|cve| Cve {
            cve_id: cve.content.clone(),
            cvss3: cve.cvss3.clone(),
            impact: cve.impact.clone(),
            href: cve.href.clone(),
            content: cve.content.clone(),
        })
        .collect()
}

/// 转换条件标准信息
///
/// # 参数
///
/// * `criteria` - OVAL条件标准信息
///
/// # 返回值
///
/// 返回转换后的数据库Criteria
pub fn convert_criteria(criteria: &oval::Criteria) -> Criteria {
    Criteria {
        operator: criteria.operator.clone(),
        criterion: convert_criterion_list(&criteria.criterion),
        sub_criteria: criteria
            .sub_criteria
            .as_ref()
            .map(|sub| sub.iter().map(convert_criteria).collect()),
    }
}

/// 转换条件信息列表
///
/// # 参数
///
/// * `criterion` - OVAL条件信息列表
///
/// # 返回值
///
/// 返回转换后的数据库Criterion列表
fn convert_criterion_list(criterion: &Vec<oval::Criterion>) -> Vec<Criterion> {
    criterion
        .iter()
        .map(|c| Criterion {
            comment: c.comment.clone(),
            test_ref: c.test_ref.clone(),
        })
        .collect()
}

/// 转换RPM信息测试列表
///
/// # 参数
///
/// * `tests` - OVAL测试信息
///
/// # 返回值
///
/// 返回转换后的数据库RpmInfoTest列表
pub fn convert_rpminfo_tests(tests: &oval::Tests) -> Vec<RpmInfoTest> {
    tests
        .rpminfo_tests
        .iter()
        .map(|t| convert_rpminfo_test(t))
        .collect()
}

/// 转换RPM信息测试
///
/// # 参数
///
/// * `test` - OVAL RPM信息测试
///
/// # 返回值
///
/// 返回转换后的数据库RpmInfoTest
fn convert_rpminfo_test(test: &oval::RpmInfoTest) -> RpmInfoTest {
    RpmInfoTest {
        check: test.check.clone(),
        comment: test.comment.clone(),
        test_id: test.id.clone(),
        version: test.version,
        object_ref: test.object.object_ref.clone(),
        state_ref: test.state.state_ref.clone(),
    }
}

/// 转换RPM信息对象列表
///
/// # 参数
///
/// * `objects` - OVAL对象信息
///
/// # 返回值
///
/// 返回转换后的数据库RpmInfoObject列表
pub fn convert_rpminfo_objects(objects: &oval::Objects) -> Vec<RpmInfoObject> {
    objects
        .rpm_info_objects
        .iter()
        .map(|o| convert_rpminfo_object(o))
        .collect()
}

/// 转换RPM信息对象
///
/// # 参数
///
/// * `object` - OVAL RPM信息对象
///
/// # 返回值
///
/// 返回转换后的数据库RpmInfoObject
fn convert_rpminfo_object(object: &oval::RpmInfoObject) -> RpmInfoObject {
    RpmInfoObject {
        id: None, // 数据库自增ID，在保存时由数据库生成
        object_id: object.id.to_string(),
        ver: object.ver,
        rpm_name: object.rpm_name.clone(),
    }
}

/// 转换RPM信息状态列表
///
/// # 参数
///
/// * `states` - OVAL状态信息
///
/// # 返回값
///
/// 返回转换后的数据库RpmInfoState列表
pub fn convert_rpminfo_states(states: &oval::States) -> Vec<RpmInfoState> {
    match &states.rpminfo_states {
        Some(s) => s.iter().map(|s| convert_rpminfo_state(s)).collect(),
        None => Vec::new(),
    }
}

/// 转换RPM信息状态
///
/// # 参数
///
/// * `state` - OVAL RPM信息状态
///
/// # 返回值
///
/// 返回转换后的数据库RpmInfoState
fn convert_rpminfo_state(state: &oval::RpmInfoState) -> RpmInfoState {
    RpmInfoState {
        state_id: state.id.clone(),
        version: state.version.clone(),
        evr_datatype: state.evr.as_ref().map(|e| e.datatype.clone()),
        evr_operation: state.evr.as_ref().map(|e| e.operation.clone()),
        evr_value: state.evr.as_ref().map(|e| e.evr.clone()),
    }
}
