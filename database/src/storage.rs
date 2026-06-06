//! 数据库存储模块
//!
//! 该模块提供了数据库存储相关的功能实现。

use crate::{
    Cve, DatabaseError, DatabaseManager, OvalDefinition, Reference, RpmInfoObject, RpmInfoState,
    RpmInfoTest,
};
use log::{debug, info, warn};

impl DatabaseManager {
    /// 从软件包版本中提取dist标识
    /// 例如: "ansible-2.9-1.oe1" -> Some("oe1")
    fn extract_dist_from_package(package_version: &str) -> Option<String> {
        let exact_patterns = vec!["oe2403", "oe2203", "oe1", "el9", "el8", "el7", "ule4"];
        for pattern in exact_patterns {
            if package_version.contains(pattern) {
                debug!(
                    "从软件包版本 {} 中精确匹配到dist: {}",
                    package_version, pattern
                );
                return Some(pattern.to_string());
            }
        }
        // 模糊匹配
        if package_version.contains("oe2003") || package_version.contains("oe20.03") {
            return Some("oe1".to_string());
        }
        if package_version.contains("oe2203") || package_version.contains("oe22.03") {
            return Some("oe2203".to_string());
        }
        if package_version.contains("oe2403") || package_version.contains("oe24.03") {
            return Some("oe2403".to_string());
        }
        warn!("无法从软件包版本 {} 中提取dist标识", package_version);
        None
    }

    /// 根据dist标识查询os_info_id
    async fn get_os_info_id_by_dist(&self, dist: &str) -> Result<Option<i64>, DatabaseError> {
        debug!("根据dist查询os_info_id: {}", dist);
        let row = self
            .client
            .query_opt("SELECT id FROM os_info WHERE dist = $1", &[&dist])
            .await?;

        if let Some(row) = row {
            let id: i64 = row.get("id");
            debug!("找到os_info_id: {} for dist: {}", id, dist);
            Ok(Some(id))
        } else {
            warn!("未找到dist对应的os_info: {}", dist);
            Ok(None)
        }
    }

    /// 从RPM状态列表中提取dist并获取os_info_id
    async fn extract_os_info_id_from_states(
        &self,
        rpminfo_states: &Vec<RpmInfoState>,
    ) -> Result<Option<i64>, DatabaseError> {
        // 尝试从第一个RPM状态的EVR中提取dist
        if let Some(first_state) = rpminfo_states.first() {
            if let Some(ref evr_value) = first_state.evr_value {
                if let Some(dist) = Self::extract_dist_from_package(evr_value) {
                    return self.get_os_info_id_by_dist(&dist).await;
                }
            }
        }

        // 如果第一个状态无法提取,尝试其他状态
        for state in rpminfo_states {
            if let Some(ref evr_value) = state.evr_value {
                if let Some(dist) = Self::extract_dist_from_package(evr_value) {
                    return self.get_os_info_id_by_dist(&dist).await;
                }
            }
        }

        Ok(None)
    }

    /// 保存完整的OVAL定义到数据库（包括所有子项目）
    pub async fn save_full_oval_definition(
        &mut self,
        definition: &OvalDefinition,
        references: &Vec<Reference>,
        cves: &Vec<Cve>,
        rpminfo_tests: &Vec<RpmInfoTest>,
        rpminfo_objects: &Vec<RpmInfoObject>,
        rpminfo_states: &Vec<RpmInfoState>,
    ) -> Result<(), DatabaseError> {
        info!("正在保存OVAL定义到数据库: {}", definition.id);
        debug!(
            "保存的OVAL定义详情: title={}, references_count={}, cves_count={}",
            definition.title,
            references.len(),
            cves.len()
        );

        // 如果definition的os_info_id为None,尝试从RPM状态中提取
        let final_definition = if definition.os_info_id.is_none() {
            let os_info_id = self.extract_os_info_id_from_states(rpminfo_states).await?;
            if let Some(id) = os_info_id {
                info!("自动匹配到os_info_id: {}", id);
                let mut updated_def = definition.clone();
                updated_def.os_info_id = Some(id);
                updated_def
            } else {
                warn!("无法自动匹配os_info_id,将保存为None");
                definition.clone()
            }
        } else {
            definition.clone()
        };

        // 开始事务
        let transaction = self.client.transaction().await?;

        // 保存OVAL定义主信息（使用可能已更新的definition）
        transaction
            .execute(
                "INSERT INTO oval_definitions (
                id, class, version, title, description, family, platform,
                severity, rights, from_field, issued_date, updated_date, os_info_id
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            ON CONFLICT (id) DO UPDATE SET
                class = EXCLUDED.class,
                version = EXCLUDED.version,
                title = EXCLUDED.title,
                description = EXCLUDED.description,
                family = EXCLUDED.family,
                platform = EXCLUDED.platform,
                severity = EXCLUDED.severity,
                rights = EXCLUDED.rights,
                from_field = EXCLUDED.from_field,
                issued_date = EXCLUDED.issued_date,
                updated_date = EXCLUDED.updated_date,
                os_info_id = EXCLUDED.os_info_id",
                &[
                    &final_definition.id,
                    &final_definition.class,
                    &(final_definition.version as i32),
                    &final_definition.title,
                    &final_definition.description,
                    &final_definition.family,
                    &final_definition.platform,
                    &final_definition.severity,
                    &final_definition.rights,
                    &final_definition.from,
                    &final_definition.issued_date,
                    &final_definition.updated_date,
                    &final_definition.os_info_id,
                ],
            )
            .await?;

        // 保存引用信息（使用final_definition的id）
        for reference in references {
            transaction
                .execute(
                    "INSERT INTO references_info (
                    oval_definition_id, ref_id, ref_url, source
                ) VALUES ($1, $2, $3, $4)
                ON CONFLICT DO NOTHING",
                    &[
                        &final_definition.id,
                        &reference.ref_id,
                        &reference.ref_url,
                        &reference.source,
                    ],
                )
                .await?;
        }

        // 保存CVE信息
        for cve in cves {
            transaction
                .execute(
                    "INSERT INTO cves (
                    oval_definition_id, cve_id, cvss3, impact, href, content
                ) VALUES ($1, $2, $3, $4, $5, $6)
                ON CONFLICT DO NOTHING",
                    &[
                        &final_definition.id,
                        &cve.cve_id,
                        &cve.cvss3,
                        &cve.impact,
                        &cve.href,
                        &cve.content,
                    ],
                )
                .await?;
        }

        // 保存RPM信息测试
        for rpminfo_test in rpminfo_tests {
            transaction.execute(
                "INSERT INTO rpminfo_tests (
                    oval_definition_id, check_field, comment, test_id, version, object_ref, state_ref
                ) VALUES ($1, $2, $3, $4, $5, $6, $7)
                ON CONFLICT (oval_definition_id, test_id) DO UPDATE SET
                    check_field = EXCLUDED.check_field,
                    comment = EXCLUDED.comment,
                    version = EXCLUDED.version,
                    object_ref = EXCLUDED.object_ref,
                    state_ref = EXCLUDED.state_ref",
                &[
                    &final_definition.id,
                    &rpminfo_test.check,
                    &rpminfo_test.comment,
                    &rpminfo_test.test_id,
                    &(rpminfo_test.version as i32),
                    &rpminfo_test.object_ref,
                    &rpminfo_test.state_ref,
                ]
            ).await?;
        }

        // 保存RPM信息对象
        for rpminfo_object in rpminfo_objects {
            transaction
                .execute(
                    "INSERT INTO rpminfo_objects (
                    oval_definition_id, object_id, ver, rpm_name
                ) VALUES ($1, $2, $3, $4)
                ON CONFLICT (oval_definition_id, object_id) DO UPDATE SET
                    ver = EXCLUDED.ver,
                    rpm_name = EXCLUDED.rpm_name",
                    &[
                        &final_definition.id,
                        &rpminfo_object.object_id,
                        &(rpminfo_object.ver as i64),
                        &rpminfo_object.rpm_name,
                    ],
                )
                .await?;
        }

        // 保存RPM信息状态（包含EVR信息）
        for rpminfo_state in rpminfo_states {
            transaction
                .execute(
                    "INSERT INTO rpminfo_states (
                    state_id, oval_definition_id, version, evr_datatype, evr_operation, evr_value
                ) VALUES ($1, $2, $3, $4, $5, $6)
                ON CONFLICT (oval_definition_id, state_id) DO UPDATE SET
                    version = EXCLUDED.version,
                    evr_datatype = EXCLUDED.evr_datatype,
                    evr_operation = EXCLUDED.evr_operation,
                    evr_value = EXCLUDED.evr_value",
                    &[
                        &rpminfo_state.state_id,
                        &final_definition.id,
                        &rpminfo_state.version,
                        &rpminfo_state.evr_datatype,
                        &rpminfo_state.evr_operation,
                        &rpminfo_state.evr_value,
                    ],
                )
                .await?;
        }

        // 提交事务
        transaction.commit().await?;

        info!(
            "OVAL定义保存成功: {}, os_info_id: {:?}",
            final_definition.id, final_definition.os_info_id
        );
        Ok(())
    }
}
