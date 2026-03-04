//! 数据库查询模块
//!
//! 该模块提供了数据库查询相关的功能实现。

use crate::{
    Criteria, Criterion, Cve, DatabaseError, DatabaseManager, OsInfo, OvalDefinition, Reference,
    RpmInfoObject, RpmInfoState, RpmInfoTest,
};
use log::{debug, error, info};

impl DatabaseManager {
    /// 根据ID获取完整的OVAL定义（包括所有子项目）
    pub async fn get_full_oval_definition(
        &self,
        id: &str,
    ) -> Result<
        Option<(
            OvalDefinition,
            Vec<Reference>,
            Vec<Cve>,
            Vec<RpmInfoTest>,
            Vec<RpmInfoObject>,
            Vec<RpmInfoState>,
        )>,
        DatabaseError,
    > {
        info!("正在从数据库获取OVAL定义: {}", id);

        // 获取OVAL定义主信息
        let definition = match self.get_oval_definition(id).await? {
            Some(def) => def,
            None => {
                info!("未找到OVAL定义: {}", id);
                return Ok(None);
            }
        };

        // 获取引用信息
        let references = self.get_references_for_definition(id).await?;

        // 获取CVE信息
        let cves = self.get_cves_for_definition(id).await?;

        // 获取RPM信息测试
        let rpminfo_tests = self.get_rpminfo_tests_for_definition(id).await?;

        // 获取RPM信息对象
        let rpminfo_objects = self.get_rpminfo_objects_for_definition(id).await?;

        // 获取RPM信息状态
        let rpminfo_states = self.get_rpminfo_states_for_definition(id).await?;

        info!(
            "成功获取OVAL定义: {}, references: {}, cves: {}, tests: {}, objects: {}, states: {}",
            id,
            references.len(),
            cves.len(),
            rpminfo_tests.len(),
            rpminfo_objects.len(),
            rpminfo_states.len()
        );

        Ok(Some((
            definition,
            references,
            cves,
            rpminfo_tests,
            rpminfo_objects,
            rpminfo_states,
        )))
    }

    /// 根据ID获取OVAL定义并转换为XML字符串
    pub async fn get_oval_xml_by_id(&self, id: &str) -> Result<Option<String>, DatabaseError> {
        info!("正在从数据库获取OVAL定义并转换为XML: {}", id);

        // 获取完整的OVAL定义
        let full_definition = match self.get_full_oval_definition(id).await? {
            Some(def) => def,
            None => {
                info!("未找到OVAL定义: {}", id);
                return Ok(None);
            }
        };

        let (definition, references, cves, rpminfo_tests, rpminfo_objects, rpminfo_states) =
            full_definition;

        // 转换为OVAL格式
        let oval_definition = self
            .convert_to_oval_definition(
                &definition,
                &references,
                &cves,
                &rpminfo_tests,
                &rpminfo_objects,
                &rpminfo_states,
            )
            .await?;

        // 转换为XML字符串
        match oval_definition.to_oval_string() {
            Ok(xml) => {
                info!("成功将OVAL定义转换为XML字符串");
                Ok(Some(xml))
            }
            Err(e) => {
                error!("转换OVAL定义为XML字符串失败: {}", e);
                // 直接使用from转换错误
                Err(DatabaseError::from(serde_json::Error::io(
                    std::io::Error::new(std::io::ErrorKind::Other, format!("OVAL转换失败: {}", e)),
                )))
            }
        }
    }

    /// 将数据库实体转换为OVAL定义
    async fn convert_to_oval_definition(
        &self,
        definition: &OvalDefinition,
        references: &Vec<Reference>,
        cves: &Vec<Cve>,
        rpminfo_tests: &Vec<RpmInfoTest>,
        rpminfo_objects: &Vec<RpmInfoObject>,
        rpminfo_states: &Vec<RpmInfoState>,
    ) -> Result<oval::OvalDefinitions, DatabaseError> {
        info!("正在将数据库实体转换为OVAL定义");

        let mut oval = oval::OvalDefinitions::new();

        // 设置时间戳（使用UTC时间格式，符合xs:dateTime要求）
        let now = chrono::Utc::now();
        oval.generator.time_stamp = now.to_rfc3339();

        // 创建OVAL定义
        let mut oval_definition = oval::Definition::new();
        oval_definition.id = definition.id.clone();
        oval_definition.class = definition.class.clone();
        oval_definition.version = definition.version;

        // 创建元数据
        let mut metadata = oval::Metadata::new();
        metadata.title = definition.title.clone();
        metadata.description = definition.description.clone();

        // 创建影响范围
        let mut affected = oval::Affected::new();
        affected.family = definition.family.clone();
        affected.platform = definition.platform.clone();
        metadata.affected = affected;

        // 创建引用列表
        let oval_references: Vec<oval::Reference> = references
            .iter()
            .map(|r| {
                let mut ref_item = oval::Reference::new();
                ref_item.ref_id = r.ref_id.clone();
                ref_item.ref_url = r.ref_url.clone();
                ref_item.source = r.source.clone();
                ref_item
            })
            .collect();
        metadata.references = Some(oval_references);

        // 创建建议信息
        let mut advisory = oval::Advisory::new();
        advisory.from = definition.from.clone();
        advisory.severity = definition.severity.clone();
        advisory.rights = definition.rights.clone();

        let mut issued = oval::Issued::new();
        issued.date = definition.issued_date.clone();
        advisory.issued = issued;

        let mut updated = oval::Updated::new();
        updated.date = definition.updated_date.clone();
        advisory.updated = updated;

        // 创建CVE列表
        let oval_cves: Vec<oval::CVE> = cves
            .iter()
            .map(|c| {
                let mut cve = oval::CVE::new();
                cve.cvss3 = c.cvss3.clone();
                cve.href = c.href.clone();
                cve.impact = c.impact.clone();
                cve.content = c.content.clone();
                cve
            })
            .collect();
        advisory.cve = oval_cves;

        metadata.advisory = advisory;
        oval_definition.metadata = metadata;

        // 参考parser crate从csaf生成criteria的逻辑重新组装criteria
        // 构建criteria的嵌套结构：
        // 1. 最外层：OR条件，包含操作系统必须安装的criterion
        // 2. 第二层：AND条件，包含操作系统已安装的criterion
        // 3. 第三层：OR条件，包含所有软件包的AND条件
        // 4. 最内层：每个软件包的AND条件（虽然目前只有一个criterion）

        let mut criteria = oval::Criteria::new();

        // 根据os_info_id获取OS信息并生成OS检查组件
        let mut os_tests: Vec<oval::RpmVerifyFileTest> = Vec::new();
        let mut os_objects: Vec<oval::RpmVerifyFileObject> = Vec::new();
        let mut os_states: Vec<oval::RpmVerifyFileState> = Vec::new();

        if !rpminfo_tests.is_empty() {
            // 为每个RPM信息测试创建独立的AND criteria
            let mut pkg_and_criterions: Vec<oval::Criteria> = Vec::new();

            for test in rpminfo_tests {
                let pkg_criteria = oval::Criteria {
                    operator: "AND".to_string(),
                    criterion: vec![oval::Criterion {
                        comment: test.comment.clone(),
                        test_ref: test.test_id.clone(),
                    }],
                    sub_criteria: None,
                };
                pkg_and_criterions.push(pkg_criteria);
            }

            // 创建包含所有软件包条件的OR criteria
            let pkg_or_criteria = oval::Criteria {
                operator: "OR".to_string(),
                criterion: Vec::new(),
                sub_criteria: Some(pkg_and_criterions),
            };

            // 根据os_info_id生成OS检查组件
            if let Some(os_info_id) = definition.os_info_id {
                if let Ok(Some(os_info)) = self.get_os_info_by_id(os_info_id).await {
                    info!("找到OS信息: {} {}", os_info.os_type, os_info.os_version);

                    // 使用共享的 ID 生成函数
                    let (
                        os_object_id,
                        os_state_full_id,
                        os_state_name_only_id,
                        os_test_must_id,
                        os_test_is_id,
                    ) = crate::generate_os_check_ids(os_info_id);

                    // 创建RpmVerifyFileObject
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
                        filepath: os_info.verify_file.clone(),
                    };
                    os_objects.push(os_object);

                    // 创建两个RpmVerifyFileState
                    // State 1: 完整检查 (name + version) - 用于 "must be installed"
                    // version 字段使用 os_version 进行匹配
                    // 转义特殊字符（如 . 转为 \\.)
                    let version_match_pattern =
                        format!("^{}", os_info.os_version);
                    let os_state_full = oval::RpmVerifyFileState {
                        id: os_state_full_id.clone(),
                        version: "1".to_string(),
                        name: oval::StateData {
                            operation: "pattern match".to_string(),
                            content: os_info.package_name.clone(),
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
                            content: os_info.package_name.clone(),
                        },
                        os_version: None, // 不检查版本
                    };
                    os_states.push(os_state_name_only);

                    // 创建两个RpmVerifyFileTest
                    // Test 1: "must be installed" - 使用仅检查名称 state
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

                    // Test 2: "is installed" - 使用完整检查 state
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

                    // 使用OS信息创建criteria
                    let platform = format!("{} {}", os_info.os_type, os_info.os_version);

                    // 创建操作系统已安装的AND criteria
                    let os_and_criteria = oval::Criteria {
                        operator: "AND".to_string(),
                        criterion: vec![oval::Criterion {
                            comment: format!("{} is installed", platform),
                            test_ref: os_test_is_id,
                        }],
                        sub_criteria: Some(vec![pkg_or_criteria]),
                    };

                    // 创建最外层的OR criteria
                    // "must be installed" 只使用 os_type，不包含版本号
                    criteria = oval::Criteria {
                        operator: "OR".to_string(),
                        criterion: vec![oval::Criterion {
                            comment: format!("{} must be installed", os_info.os_type),
                            test_ref: os_test_must_id,
                        }],
                        sub_criteria: Some(vec![os_and_criteria]),
                    };
                } else {
                    // 如果没有找到OS信息，使用os_info_id=0（Unknown）生成ID
                    let (_, _, _, os_test_must_id_unknown, os_test_is_id_unknown) =
                        crate::generate_os_check_ids(0);

                    let platform = if !definition.platform.is_empty() {
                        definition.platform.clone()
                    } else {
                        "operating system".to_string()
                    };

                    // 创建操作系统已安装的AND criteria
                    let os_and_criteria = oval::Criteria {
                        operator: "AND".to_string(),
                        criterion: vec![oval::Criterion {
                            comment: format!("{} is installed", platform),
                            test_ref: os_test_is_id_unknown,
                        }],
                        sub_criteria: Some(vec![pkg_or_criteria]),
                    };

                    // 创建最外层的OR criteria
                    criteria = oval::Criteria {
                        operator: "OR".to_string(),
                        criterion: vec![oval::Criterion {
                            comment: format!("{} must be installed", platform),
                            test_ref: os_test_must_id_unknown,
                        }],
                        sub_criteria: Some(vec![os_and_criteria]),
                    };
                }
            } else {
                // 如果没有os_info_id，使用os_info_id=0（Unknown）生成ID
                let (_, _, _, os_test_must_id_unknown, os_test_is_id_unknown) =
                    crate::generate_os_check_ids(0);

                let platform = if !definition.platform.is_empty() {
                    definition.platform.clone()
                } else {
                    "operating system".to_string()
                };

                // 创建操作系统已安装的AND criteria
                let os_and_criteria = oval::Criteria {
                    operator: "AND".to_string(),
                    criterion: vec![oval::Criterion {
                        comment: format!("{} is installed", platform),
                        test_ref: os_test_is_id_unknown,
                    }],
                    sub_criteria: Some(vec![pkg_or_criteria]),
                };

                // 创建最外层的OR criteria
                criteria = oval::Criteria {
                    operator: "OR".to_string(),
                    criterion: vec![oval::Criterion {
                        comment: format!("{} must be installed", platform),
                        test_ref: os_test_must_id_unknown,
                    }],
                    sub_criteria: Some(vec![os_and_criteria]),
                };
            }
        } else {
            // 如果没有测试，创建一个空的AND criteria
            criteria.operator = "AND".to_string();
        }

        oval_definition.criteria = criteria;

        // 添加到定义列表
        let mut definitions = oval::Definitions::new();
        definitions.items.push(oval_definition);
        oval.definitions = definitions;

        // 创建测试列表
        let oval_tests: Vec<oval::RpmInfoTest> = rpminfo_tests
            .iter()
            .map(|t| {
                let mut test = oval::RpmInfoTest::new();
                test.check = t.check.clone();
                test.comment = t.comment.clone();
                test.id = t.test_id.clone();
                test.version = t.version;

                let mut object_ref = oval::ObjectReference::new();
                object_ref.object_ref = t.object_ref.clone();
                test.object = object_ref;

                let mut state_ref = oval::StateReference::new();
                state_ref.state_ref = t.state_ref.clone();
                test.state = state_ref;

                test
            })
            .collect();
        oval.tests.rpminfo_tests = oval_tests;

        // 创建对象列表
        let oval_objects: Vec<oval::RpmInfoObject> = rpminfo_objects
            .iter()
            .map(|o| {
                let mut object = oval::RpmInfoObject::new();
                object.id = o.object_id.clone();
                object.ver = o.ver;
                object.rpm_name = o.rpm_name.clone();
                object
            })
            .collect();
        oval.objects.rpm_info_objects = oval_objects;

        // 创建状态列表
        let oval_states: Vec<oval::RpmInfoState> = rpminfo_states
            .iter()
            .map(|s| {
                let mut state = oval::RpmInfoState::new();
                state.id = s.state_id.clone();
                state.version = s.version.clone();

                // 如果EVR信息存在，则创建 EVR 对象
                if let (Some(datatype), Some(operation), Some(evr_value)) =
                    (&s.evr_datatype, &s.evr_operation, &s.evr_value)
                {
                    let mut oval_evr = oval::Evr::new();
                    oval_evr.datatype = datatype.clone();
                    oval_evr.operation = operation.clone();
                    oval_evr.evr = evr_value.clone();
                    state.evr = Some(oval_evr);
                }

                state
            })
            .collect();
        oval.states.rpminfo_states = Some(oval_states);

        // 添加OS检查相关的tests、objects和states
        if !os_tests.is_empty() {
            oval.tests.rpmverifyfile_tests = os_tests;
        }
        if !os_objects.is_empty() {
            oval.objects.rpmverifyfile_objects = os_objects;
        }
        if !os_states.is_empty() {
            oval.states.rpmverifyfile_states = Some(os_states);
        }

        info!("成功将数据库实体转换为OVAL定义");
        Ok(oval)
    }

    /// 根据ID获取OVAL定义（移除了oval_data字段）
    pub async fn get_oval_definition(
        &self,
        id: &str,
    ) -> Result<Option<OvalDefinition>, DatabaseError> {
        debug!("查询OVAL定义: {}", id);
        let row = self
            .client
            .query_opt(
                "SELECT id, class, version, title, description, family, platform,
                    severity, rights, from_field, issued_date, updated_date, os_info_id
             FROM oval_definitions WHERE id = $1",
                &[&id],
            )
            .await?;

        if let Some(row) = row {
            Ok(Some(OvalDefinition {
                id: row.get("id"),
                class: row.get("class"),
                version: row.get::<_, i32>("version") as u32,
                title: row.get("title"),
                description: row.get("description"),
                family: row.get("family"),
                platform: row.get("platform"),
                severity: row.get("severity"),
                rights: row.get("rights"),
                from: row.get("from_field"),
                issued_date: row.get("issued_date"),
                updated_date: row.get("updated_date"),
                os_info_id: row.get("os_info_id"),
            }))
        } else {
            Ok(None)
        }
    }

    /// 获取指定OVAL定义的引用信息
    pub async fn get_references_for_definition(
        &self,
        oval_definition_id: &str,
    ) -> Result<Vec<Reference>, DatabaseError> {
        debug!("查询引用信息: {}", oval_definition_id);
        let rows = self
            .client
            .query(
                "SELECT ref_id, ref_url, source FROM references_info WHERE oval_definition_id = $1",
                &[&oval_definition_id],
            )
            .await?;

        let mut references = Vec::new();
        for row in rows {
            references.push(Reference {
                ref_id: row.get("ref_id"),
                ref_url: row.get("ref_url"),
                source: row.get("source"),
            });
        }

        Ok(references)
    }

    /// 获取指定OVAL定义的CVE信息
    pub async fn get_cves_for_definition(
        &self,
        oval_definition_id: &str,
    ) -> Result<Vec<Cve>, DatabaseError> {
        debug!("查询CVE信息: {}", oval_definition_id);
        let rows = self.client.query(
            "SELECT cve_id, cvss3, impact, href, content FROM cves WHERE oval_definition_id = $1",
            &[&oval_definition_id]
        ).await?;

        let mut cves = Vec::new();
        for row in rows {
            cves.push(Cve {
                cve_id: row.get("cve_id"),
                cvss3: row.get("cvss3"),
                impact: row.get("impact"),
                href: row.get("href"),
                content: row.get("content"),
            });
        }

        Ok(cves)
    }

    /// 获取指定OVAL定义的条件标准信息
    pub async fn get_criteria_for_definition(
        &self,
        oval_definition_id: &str,
    ) -> Result<Criteria, DatabaseError> {
        debug!("查询条件标准信息: {}", oval_definition_id);
        let criteria_row = self.client.query_opt(
            "SELECT id, operator FROM criteria WHERE oval_definition_id = $1 ORDER BY id LIMIT 1",
            &[&oval_definition_id]
        ).await?;

        if let Some(row) = criteria_row {
            let criteria_id: i64 = row.get("id");
            let operator: String = row.get("operator");

            // 获取条件信息
            let criterion_rows = self
                .client
                .query(
                    "SELECT comment, test_ref FROM criterion WHERE criteria_id = $1",
                    &[&criteria_id],
                )
                .await?;

            let mut criterion = Vec::new();
            for row in criterion_rows {
                criterion.push(Criterion {
                    comment: row.get("comment"),
                    test_ref: row.get("test_ref"),
                });
            }

            // 简化处理，实际应该递归获取子条件
            Ok(Criteria {
                operator,
                criterion,
                sub_criteria: None,
            })
        } else {
            Ok(Criteria {
                operator: "AND".to_string(),
                criterion: vec![],
                sub_criteria: None,
            })
        }
    }

    /// 获取指定OVAL定义的RPM信息测试
    pub async fn get_rpminfo_tests_for_definition(
        &self,
        oval_definition_id: &str,
    ) -> Result<Vec<RpmInfoTest>, DatabaseError> {
        debug!("查询RPM信息测试: {}", oval_definition_id);
        let rows = self
            .client
            .query(
                "SELECT check_field, comment, test_id, version, object_ref, state_ref
             FROM rpminfo_tests WHERE oval_definition_id = $1",
                &[&oval_definition_id],
            )
            .await?;

        let mut rpminfo_tests = Vec::new();
        for row in rows {
            rpminfo_tests.push(RpmInfoTest {
                check: row.get("check_field"),
                comment: row.get("comment"),
                test_id: row.get("test_id"),
                version: row.get::<_, i32>("version") as u32,
                object_ref: row.get("object_ref"),
                state_ref: row.get("state_ref"),
            });
        }

        Ok(rpminfo_tests)
    }

    /// 获取指定OVAL定义的RPM信息对象
    pub async fn get_rpminfo_objects_for_definition(
        &self,
        oval_definition_id: &str,
    ) -> Result<Vec<RpmInfoObject>, DatabaseError> {
        debug!("查询RPM信息对象: {}", oval_definition_id);
        let rows = self
            .client
            .query(
                "SELECT id, object_id, ver, rpm_name
             FROM rpminfo_objects WHERE oval_definition_id = $1",
                &[&oval_definition_id],
            )
            .await?;

        let mut rpm_info_objects = Vec::new();
        for row in rows {
            rpm_info_objects.push(RpmInfoObject {
                id: Some(row.get("id")), // 数据库自增ID
                object_id: row.get("object_id"),
                ver: row.get::<_, i64>("ver") as u64,
                rpm_name: row.get("rpm_name"),
            });
        }

        Ok(rpm_info_objects)
    }

    /// 获取指定OVAL定义的RPM信息状态
    pub async fn get_rpminfo_states_for_definition(
        &self,
        oval_definition_id: &str,
    ) -> Result<Vec<RpmInfoState>, DatabaseError> {
        debug!("查询RPM信息状态: {}", oval_definition_id);
        let rows = self
            .client
            .query(
                "SELECT state_id, version, evr_datatype, evr_operation, evr_value
             FROM rpminfo_states WHERE oval_definition_id = $1",
                &[&oval_definition_id],
            )
            .await?;

        let mut rpminfo_states = Vec::new();
        for row in rows {
            rpminfo_states.push(RpmInfoState {
                state_id: row.get("state_id"),
                version: row.get("version"),
                evr_datatype: row.get("evr_datatype"),
                evr_operation: row.get("evr_operation"),
                evr_value: row.get("evr_value"),
            });
        }

        Ok(rpminfo_states)
    }

    /// 列出所有OVAL定义
    pub async fn list_all_oval_definitions(&self) -> Result<Vec<OvalDefinition>, DatabaseError> {
        info!("正在查询所有OVAL定义");
        let rows = self
            .client
            .query(
                "SELECT id, class, version, title, description, family, platform,
                    severity, rights, from_field, issued_date, updated_date, os_info_id
             FROM oval_definitions ORDER BY id",
                &[],
            )
            .await?;

        let mut definitions = Vec::new();
        for row in rows {
            definitions.push(OvalDefinition {
                id: row.get("id"),
                class: row.get("class"),
                version: row.get::<_, i32>("version") as u32,
                title: row.get("title"),
                description: row.get("description"),
                family: row.get("family"),
                platform: row.get("platform"),
                severity: row.get("severity"),
                rights: row.get("rights"),
                from: row.get("from_field"),
                issued_date: row.get("issued_date"),
                updated_date: row.get("updated_date"),
                os_info_id: row.get("os_info_id"),
            });
        }

        Ok(definitions)
    }

    /// 根据dist字符串查找OS信息
    pub async fn find_os_info_by_dist(&self, dist: &str) -> Result<Option<OsInfo>, DatabaseError> {
        debug!("根据dist查找OS信息: {}", dist);
        let row = self.client.query_opt(
            "SELECT id, os_type, os_version, package_name, verify_file, verify_pattern, dist, description
             FROM os_info WHERE dist = $1",
            &[&dist]
        ).await?;

        if let Some(row) = row {
            Ok(Some(OsInfo {
                id: Some(row.get("id")),
                os_type: row.get("os_type"),
                os_version: row.get("os_version"),
                package_name: row.get("package_name"),
                verify_file: row.get("verify_file"),
                verify_pattern: row.get("verify_pattern"),
                dist: row.get("dist"),
                description: row.get("description"),
            }))
        } else {
            Ok(None)
        }
    }

    /// 根据ID查找OS信息
    pub async fn get_os_info_by_id(&self, id: i64) -> Result<Option<OsInfo>, DatabaseError> {
        debug!("根据ID查找OS信息: {}", id);
        let row = self.client.query_opt(
            "SELECT id, os_type, os_version, package_name, verify_file, verify_pattern, dist, description
             FROM os_info WHERE id = $1",
            &[&id]
        ).await?;

        if let Some(row) = row {
            Ok(Some(OsInfo {
                id: Some(row.get("id")),
                os_type: row.get("os_type"),
                os_version: row.get("os_version"),
                package_name: row.get("package_name"),
                verify_file: row.get("verify_file"),
                verify_pattern: row.get("verify_pattern"),
                dist: row.get("dist"),
                description: row.get("description"),
            }))
        } else {
            Ok(None)
        }
    }

    /// 从软件包版本字符串中提取dist并匹配OS信息
    /// 例如: ansible-2.9-1.oe1 -> oe1
    pub async fn extract_and_match_os_info(
        &self,
        package_version: &str,
    ) -> Result<Option<OsInfo>, DatabaseError> {
        debug!("从软件包版本提取dist: {}", package_version);

        // 获取所有的dist列表
        let rows = self.client.query(
            "SELECT id, os_type, os_version, package_name, verify_file, verify_pattern, dist, description
             FROM os_info ORDER BY LENGTH(dist) DESC",  // 按dist长度降序，优先匹配长的
            &[]
        ).await?;

        // 遍历所有dist，看是否包含在package_version中
        for row in rows {
            let dist: String = row.get("dist");
            if package_version.contains(&dist) {
                info!("匹配到OS信息: dist={}", dist);
                return Ok(Some(OsInfo {
                    id: Some(row.get("id")),
                    os_type: row.get("os_type"),
                    os_version: row.get("os_version"),
                    package_name: row.get("package_name"),
                    verify_file: row.get("verify_file"),
                    verify_pattern: row.get("verify_pattern"),
                    dist: row.get("dist"),
                    description: row.get("description"),
                }));
            }
        }

        debug!("未找到匹配的OS信息");
        Ok(None)
    }

    /// 列出所有OS信息
    pub async fn list_all_os_info(&self) -> Result<Vec<OsInfo>, DatabaseError> {
        info!("正在查询所有OS信息");
        let rows = self.client.query(
            "SELECT id, os_type, os_version, package_name, verify_file, verify_pattern, dist, description
             FROM os_info ORDER BY os_type, os_version",
            &[]
        ).await?;

        let mut os_infos = Vec::new();
        for row in rows {
            os_infos.push(OsInfo {
                id: Some(row.get("id")),
                os_type: row.get("os_type"),
                os_version: row.get("os_version"),
                package_name: row.get("package_name"),
                verify_file: row.get("verify_file"),
                verify_pattern: row.get("verify_pattern"),
                dist: row.get("dist"),
                description: row.get("description"),
            });
        }

        Ok(os_infos)
    }

    /// 根据多个ID导出并合并为单个OVAL定义
    ///
    /// # 参数
    ///
    /// * `definition_ids` - OVAL定义ID列表
    ///
    /// # 返回值
    ///
    /// 返回Result<oval::OvalDefinitions>，成功时包含合并后的OVAL定义
    pub async fn export_merged_oval(
        &self,
        definition_ids: Vec<String>,
    ) -> Result<oval::OvalDefinitions, DatabaseError> {
        info!("正在导出并合并 {} 个OVAL定义", definition_ids.len());

        let mut oval_list = Vec::new();

        for def_id in definition_ids {
            match self.get_full_oval_definition(&def_id).await? {
                Some(full_def) => {
                    let (definition, references, cves, rpminfo_tests, rpminfo_objects, rpminfo_states) =
                        full_def;
                    let oval_def = self
                        .convert_to_oval_definition(
                            &definition,
                            &references,
                            &cves,
                            &rpminfo_tests,
                            &rpminfo_objects,
                            &rpminfo_states,
                        )
                        .await?;
                    oval_list.push(oval_def);
                }
                None => {
                    debug!("跳过未找到的定义: {}", def_id);
                }
            }
        }

        if oval_list.is_empty() {
            info!("未找到任何有效的OVAL定义");
            return Ok(oval::OvalDefinitions::new());
        }

        info!("成功导出 {} 个OVAL定义，开始合并", oval_list.len());
        oval::OvalDefinitions::merge_multiple(oval_list)
            .map_err(|e| DatabaseError::from(serde_json::Error::io(
                std::io::Error::new(std::io::ErrorKind::Other, format!("合并OVAL失败: {}", e)),
            )))
    }

    /// 根据多个ID导出合并的OVAL XML字符串
    ///
    /// # 参数
    ///
    /// * `definition_ids` - OVAL定义ID列表
    ///
    /// # 返回值
    ///
    /// 返回Result<String>，成功时包含合并后的OVAL XML字符串
    pub async fn export_merged_oval_xml(
        &self,
        definition_ids: Vec<String>,
    ) -> Result<String, DatabaseError> {
        info!("正在导出并合并为XML，定义数量: {}", definition_ids.len());
        let merged = self.export_merged_oval(definition_ids).await?;
        merged.to_oval_string()
            .map_err(|e| DatabaseError::from(serde_json::Error::io(
                std::io::Error::new(std::io::ErrorKind::Other, format!("转换为XML失败: {}", e)),
            )))
    }

    /// 导出所有OVAL定义到单个合并的定义
    ///
    /// # 返回值
    ///
    /// 返回Result<oval::OvalDefinitions>，成功时包含所有OVAL定义的合并结果
    pub async fn export_all_oval_definitions(&self) -> Result<oval::OvalDefinitions, DatabaseError> {
        info!("正在导出所有OVAL定义");

        // 查询所有定义ID
        let definitions = self.list_all_oval_definitions().await?;
        let definition_ids: Vec<String> = definitions.iter().map(|d| d.id.clone()).collect();

        info!("找到 {} 个OVAL定义", definition_ids.len());
        self.export_merged_oval(definition_ids).await
    }

    /// 根据时间范围导出OVAL定义
    ///
    /// # 参数
    ///
    /// * `start_date` - 开始日期（格式：YYYY-MM-DD）
    /// * `end_date` - 结束日期（格式：YYYY-MM-DD）
    ///
    /// # 返回值
    ///
    /// 返回Result<oval::OvalDefinitions>，成功时包含时间范围内的OVAL定义
    pub async fn export_oval_by_date_range(
        &self,
        start_date: &str,
        end_date: &str,
    ) -> Result<oval::OvalDefinitions, DatabaseError> {
        info!("正在导出 {} 到 {} 之间的OVAL定义", start_date, end_date);

        let rows = self.client.query(
            "SELECT id FROM oval_definitions
             WHERE issued_date >= $1 AND issued_date < $2
             ORDER BY issued_date",
            &[&start_date, &end_date]
        ).await?;

        let definition_ids: Vec<String> = rows.iter().map(|row| row.get("id")).collect();

        info!("在指定时间范围内找到 {} 个OVAL定义", definition_ids.len());
        self.export_merged_oval(definition_ids).await
    }

    /// 核心通用方法：按时间和系统类型组合查询
    ///
    /// # 参数
    ///
    /// * `start_date` - 开始日期（格式：YYYY-MM-DD）
    /// * `end_date` - 结束日期（格式：YYYY-MM-DD）
    /// * `os_type` - 操作系统类型（可选），支持按dist或os_type匹配
    ///
    /// # 返回值
    ///
    /// 返回Result<oval::OvalDefinitions>，成功时包含符合条件的OVAL定义
    pub async fn export_oval_by_time_and_os(
        &self,
        start_date: &str,
        end_date: &str,
        os_type: Option<&str>,
    ) -> Result<oval::OvalDefinitions, DatabaseError> {
        info!(
            "正在导出 {} 到 {} 之间的OVAL定义，系统类型过滤: {:?}",
            start_date, end_date, os_type
        );

        let rows = if let Some(os_filter) = os_type {
            // 带系统类型过滤的查询
            self.client.query(
                "SELECT od.id
                 FROM oval_definitions od
                 LEFT JOIN os_info oi ON od.os_info_id = oi.id
                 WHERE od.issued_date >= $1
                   AND od.issued_date < $2
                   AND (
                     LOWER(oi.dist) = LOWER($3)
                     OR LOWER(oi.os_type) LIKE '%' || LOWER($3) || '%'
                   )
                 ORDER BY od.issued_date",
                &[&start_date, &end_date, &os_filter]
            ).await?
        } else {
            // 不过滤系统类型
            self.client.query(
                "SELECT id FROM oval_definitions
                 WHERE issued_date >= $1 AND issued_date < $2
                 ORDER BY issued_date",
                &[&start_date, &end_date]
            ).await?
        };

        let definition_ids: Vec<String> = rows.iter().map(|row| row.get("id")).collect();

        info!(
            "在指定时间范围内找到 {} 个OVAL定义（系统过滤: {:?}）",
            definition_ids.len(), os_type
        );
        self.export_merged_oval(definition_ids).await
    }

    /// 按月导出OVAL定义（支持系统类型过滤）
    ///
    /// # 参数
    ///
    /// * `year` - 年份
    /// * `month` - 月份（1-12）
    /// * `os_type` - 操作系统类型（可选）
    ///
    /// # 返回值
    ///
    /// 返回Result<oval::OvalDefinitions>，成功时包含指定月份的OVAL定义
    pub async fn export_oval_by_month(
        &self,
        year: i32,
        month: u32,
        os_type: Option<&str>,
    ) -> Result<oval::OvalDefinitions, DatabaseError> {
        info!("正在导出 {}-{:02} 的OVAL定义，系统类型过滤: {:?}", year, month, os_type);

        // 计算月份的开始和结束日期
        let start_date = format!("{}-{:02}-01", year, month);
        let end_date = if month == 12 {
            format!("{}-01-01", year + 1)
        } else {
            format!("{}-{:02}-01", year, month + 1)
        };

        self.export_oval_by_time_and_os(&start_date, &end_date, os_type).await
    }

    /// 按周导出OVAL定义（支持系统类型过滤）
    ///
    /// # 参数
    ///
    /// * `year` - 年份
    /// * `week` - ISO周号（1-53）
    /// * `os_type` - 操作系统类型（可选）
    ///
    /// # 返回值
    ///
    /// 返回Result<oval::OvalDefinitions>，成功时包含指定周的OVAL定义
    pub async fn export_oval_by_week(
        &self,
        year: i32,
        week: u32,
        os_type: Option<&str>,
    ) -> Result<oval::OvalDefinitions, DatabaseError> {
        info!("正在导出 {}-W{:02} 的OVAL定义，系统类型过滤: {:?}", year, week, os_type);

        let rows = if let Some(os_filter) = os_type {
            // 带系统类型过滤的查询
            self.client.query(
                "SELECT od.id
                 FROM oval_definitions od
                 LEFT JOIN os_info oi ON od.os_info_id = oi.id
                 WHERE EXTRACT(ISOYEAR FROM od.issued_date::date) = $1
                   AND EXTRACT(WEEK FROM od.issued_date::date) = $2
                   AND (
                     LOWER(oi.dist) = LOWER($3)
                     OR LOWER(oi.os_type) LIKE '%' || LOWER($3) || '%'
                   )
                 ORDER BY od.issued_date",
                &[&year, &(week as i64), &os_filter]
            ).await?
        } else {
            // 不过滤系统类型
            self.client.query(
                "SELECT id FROM oval_definitions
                 WHERE EXTRACT(ISOYEAR FROM issued_date::date) = $1
                   AND EXTRACT(WEEK FROM issued_date::date) = $2
                 ORDER BY issued_date",
                &[&year, &(week as i64)]
            ).await?
        };

        let definition_ids: Vec<String> = rows.iter().map(|row| row.get("id")).collect();

        info!(
            "在指定周范围内找到 {} 个OVAL定义（系统过滤: {:?}）",
            definition_ids.len(), os_type
        );
        self.export_merged_oval(definition_ids).await
    }

    /// 按年导出OVAL定义（支持系统类型过滤）
    ///
    /// # 参数
    ///
    /// * `year` - 年份
    /// * `os_type` - 操作系统类型（可选）
    ///
    /// # 返回值
    ///
    /// 返回Result<oval::OvalDefinitions>，成功时包含指定年份的OVAL定义
    pub async fn export_oval_by_year(
        &self,
        year: i32,
        os_type: Option<&str>,
    ) -> Result<oval::OvalDefinitions, DatabaseError> {
        info!("正在导出 {} 年的OVAL定义，系统类型过滤: {:?}", year, os_type);

        // 计算年份的开始和结束日期
        let start_date = format!("{}-01-01", year);
        let end_date = format!("{}-01-01", year + 1);

        self.export_oval_by_time_and_os(&start_date, &end_date, os_type).await
    }

    /// 按操作系统类型导出所有OVAL定义
    ///
    /// # 参数
    ///
    /// * `os_type` - 操作系统类型（按dist或os_type匹配）
    ///
    /// # 返回值
    ///
    /// 返回Result<oval::OvalDefinitions>，成功时包含指定系统的所有OVAL定义
    pub async fn export_oval_by_os_type(
        &self,
        os_type: &str,
    ) -> Result<oval::OvalDefinitions, DatabaseError> {
        info!("正在导出操作系统类型 {} 的所有OVAL定义", os_type);

        let rows = self.client.query(
            "SELECT od.id
             FROM oval_definitions od
             JOIN os_info oi ON od.os_info_id = oi.id
             WHERE LOWER(oi.dist) = LOWER($1)
                OR LOWER(oi.os_type) LIKE '%' || LOWER($1) || '%'
             ORDER BY od.issued_date",
            &[&os_type]
        ).await?;

        let definition_ids: Vec<String> = rows.iter().map(|row| row.get("id")).collect();

        info!("找到 {} 个操作系统类型为 {} 的OVAL定义", definition_ids.len(), os_type);
        self.export_merged_oval(definition_ids).await
    }
}
