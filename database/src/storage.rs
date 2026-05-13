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
        todo!();
    }

    /// 根据dist标识查询os_info_id
    async fn get_os_info_id_by_dist(&self, dist: &str) -> Result<Option<i64>, DatabaseError> {
        todo!()
    }

    /// 从RPM状态列表中提取dist并获取os_info_id
    async fn extract_os_info_id_from_states(
        &self,
        rpminfo_states: &Vec<RpmInfoState>,
    ) -> Result<Option<i64>, DatabaseError> {
        todo!()
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
        todo!()
    }
}
