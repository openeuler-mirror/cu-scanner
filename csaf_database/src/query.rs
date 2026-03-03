//! CSAF 数据库查询模块
//!
//! 该模块提供了从数据库查询 CSAF 相关数据的功能。

use crate::{DatabaseError, DatabaseManager, schema::*};
use log::{debug, info};
use tokio_postgres::Row;

/// CSAF 数据查询器
pub struct CsafQuery {
    db_manager: DatabaseManager,
}

impl CsafQuery {
    /// 创建新的 CSAF 查询器
    pub async fn new(db_manager: DatabaseManager) -> Result<Self, DatabaseError> {
        Ok(Self { db_manager })
    }

    /// 根据 ID 获取单个安全公告信息
    pub async fn get_sa_info_by_id(&self, id: i32) -> Result<Option<SaInfo>, DatabaseError> {
        info!("查询安全公告信息，ID: {}", id);

        let row = self.db_manager.client.query_opt(
            "SELECT id, sa_id, synopsis, summary, topic, description, severity, affected_product, affected_component, status, created_time, updated_time FROM sa_info WHERE id = $1",
            &[&id]
        ).await?;

        if let Some(row) = row {
            let sa_info = self.row_to_sa_info(&row);
            debug!("成功查询到安全公告信息，ID: {}", id);
            Ok(Some(sa_info))
        } else {
            debug!("未找到安全公告信息，ID: {}", id);
            Ok(None)
        }
    }

    /// 根据 SA ID 获取安全公告信息
    pub async fn get_sa_info_by_sa_id(&self, sa_id: &str) -> Result<Option<SaInfo>, DatabaseError> {
        info!("查询安全公告信息，SA ID: {}", sa_id);

        let row = self.db_manager.client.query_opt(
            "SELECT id, sa_id, synopsis, summary, topic, description, severity, affected_product, affected_component, status, created_time, updated_time FROM sa_info WHERE sa_id = $1",
            &[&sa_id]
        ).await?;

        if let Some(row) = row {
            let sa_info = self.row_to_sa_info(&row);
            debug!("成功查询到安全公告信息，SA ID: {}", sa_id);
            Ok(Some(sa_info))
        } else {
            debug!("未找到安全公告信息，SA ID: {}", sa_id);
            Ok(None)
        }
    }

    /// 获取所有安全公告信息
    pub async fn get_all_sa_info(&self) -> Result<Vec<SaInfo>, DatabaseError> {
        info!("查询所有安全公告信息");

        let rows = self.db_manager.client.query(
            "SELECT id, sa_id, synopsis, summary, topic, description, severity, affected_product, affected_component, status, created_time, updated_time FROM sa_info ORDER BY created_time DESC",
            &[]
        ).await?;

        let sa_info_list: Vec<SaInfo> = rows.iter().map(|row| self.row_to_sa_info(row)).collect();
        debug!("成功查询到 {} 条安全公告信息", sa_info_list.len());
        Ok(sa_info_list)
    }

    /// 获取指定时间之后的安全公告ID列表（基于created_time）
    pub async fn get_sa_ids_after_time(
        &self,
        timestamp: &str,
    ) -> Result<Vec<String>, DatabaseError> {
        info!("查询创建时间在 {} 之后的安全公告ID列表", timestamp);

        let rows = self
            .db_manager
            .client
            .query(
                "SELECT sa_id FROM sa_info WHERE created_time > $1 ORDER BY created_time ASC",
                &[&timestamp],
            )
            .await?;

        let sa_ids: Vec<String> = rows.iter().map(|row| row.get("sa_id")).collect();
        debug!("成功查询到 {} 个安全公告ID", sa_ids.len());
        Ok(sa_ids)
    }

    /// 获取指定更新时间之后的安全公告ID列表（基于updated_time）
    pub async fn get_sa_ids_after_updated_time(
        &self,
        timestamp: &str,
    ) -> Result<Vec<String>, DatabaseError> {
        info!("查询更新时间在 {} 之后的安全公告ID列表", timestamp);

        let rows = self
            .db_manager
            .client
            .query(
                "SELECT sa_id FROM sa_info WHERE updated_time > $1 ORDER BY updated_time ASC",
                &[&timestamp],
            )
            .await?;

        let sa_ids: Vec<String> = rows.iter().map(|row| row.get("sa_id")).collect();
        debug!("成功查询到 {} 个安全公告ID（基于更新时间）", sa_ids.len());
        Ok(sa_ids)
    }

    /// 根据 ID 获取 CVE 信息
    pub async fn get_cve_info_by_id(&self, id: i32) -> Result<Option<CveInfo>, DatabaseError> {
        info!("查询 CVE 信息，ID: {}", id);

        let row = self.db_manager.client.query_opt(
            "SELECT id, cve_id, description, base_severity, base_score, vector_string, cvss_version, published_date, updated_date, status, created_at, updated_at FROM cve_info WHERE id = $1",
            &[&id]
        ).await?;

        if let Some(row) = row {
            let cve_info = self.row_to_cve_info(&row);
            debug!("成功查询到 CVE 信息，ID: {}", id);
            Ok(Some(cve_info))
        } else {
            debug!("未找到 CVE 信息，ID: {}", id);
            Ok(None)
        }
    }

    /// 根据 CVE ID 获取 CVE 信息
    pub async fn get_cve_info_by_cve_id(
        &self,
        cve_id: &str,
    ) -> Result<Option<CveInfo>, DatabaseError> {
        info!("查询 CVE 信息，CVE ID: {}", cve_id);

        let row = self.db_manager.client.query_opt(
            "SELECT id, cve_id, description, base_severity, base_score, vector_string, cvss_version, published_date, updated_date, status, created_at, updated_at FROM cve_info WHERE cve_id = $1",
            &[&cve_id]
        ).await?;

        if let Some(row) = row {
            let cve_info = self.row_to_cve_info(&row);
            debug!("成功查询到 CVE 信息，CVE ID: {}", cve_id);
            Ok(Some(cve_info))
        } else {
            debug!("未找到 CVE 信息，CVE ID: {}", cve_id);
            Ok(None)
        }
    }

    /// 获取所有 CVE 信息
    pub async fn get_all_cve_info(&self) -> Result<Vec<CveInfo>, DatabaseError> {
        info!("查询所有 CVE 信息");

        let rows = self.db_manager.client.query(
            "SELECT id, cve_id, description, base_severity, base_score, vector_string, cvss_version, published_date, updated_date, status, created_at, updated_at FROM cve_info ORDER BY created_at DESC",
            &[]
        ).await?;

        let cve_info_list: Vec<CveInfo> =
            rows.iter().map(|row| self.row_to_cve_info(row)).collect();
        debug!("成功查询到 {} 条 CVE 信息", cve_info_list.len());
        Ok(cve_info_list)
    }

    /// 根据 ID 获取 OS 版本映射信息
    pub async fn get_os_version_map_by_id(
        &self,
        id: i32,
    ) -> Result<Option<OsVersionMap>, DatabaseError> {
        info!("查询 OS 版本映射信息，ID: {}", id);

        let row = self.db_manager.client.query_opt(
            "SELECT id, os_version, upstream_series, dist, release_date, end_of_life, description FROM os_version_map WHERE id = $1",
            &[&id]
        ).await?;

        if let Some(row) = row {
            let os_version_map = self.row_to_os_version_map(&row);
            debug!("成功查询到 OS 版本映射信息，ID: {}", id);
            Ok(Some(os_version_map))
        } else {
            debug!("未找到 OS 版本映射信息，ID: {}", id);
            Ok(None)
        }
    }

    /// 根据 OS 版本获取 OS 版本映射信息
    pub async fn get_os_version_map_by_version(
        &self,
        os_version: &str,
    ) -> Result<Option<OsVersionMap>, DatabaseError> {
        info!("查询 OS 版本映射信息，OS 版本: {}", os_version);

        let row = self.db_manager.client.query_opt(
            "SELECT id, os_version, upstream_series, dist, release_date, end_of_life, description FROM os_version_map WHERE os_version = $1",
            &[&os_version]
        ).await?;

        if let Some(row) = row {
            let os_version_map = self.row_to_os_version_map(&row);
            debug!("成功查询到 OS 版本映射信息，OS 版本: {}", os_version);
            Ok(Some(os_version_map))
        } else {
            debug!("未找到 OS 版本映射信息，OS 版本: {}", os_version);
            Ok(None)
        }
    }

    /// 获取所有 OS 版本映射信息
    pub async fn get_all_os_version_maps(&self) -> Result<Vec<OsVersionMap>, DatabaseError> {
        info!("查询所有 OS 版本映射信息");

        let rows = self.db_manager.client.query(
            "SELECT id, os_version, upstream_series, dist, release_date, end_of_life, description FROM os_version_map ORDER BY id",
            &[]
        ).await?;

        let os_version_map_list: Vec<OsVersionMap> = rows
            .iter()
            .map(|row| self.row_to_os_version_map(row))
            .collect();
        debug!(
            "成功查询到 {} 条 OS 版本映射信息",
            os_version_map_list.len()
        );
        Ok(os_version_map_list)
    }

    /// 根据 SA ID 和 CVE ID 获取关联信息
    pub async fn get_sa_cve_by_ids(
        &self,
        sa_id: i32,
        cve_id: i32,
    ) -> Result<Option<SaCve>, DatabaseError> {
        info!(
            "查询 SA 与 CVE 关联信息，SA ID: {}, CVE ID: {}",
            sa_id, cve_id
        );

        let row = self
            .db_manager
            .client
            .query_opt(
                "SELECT sa_id, cve_id FROM sa_cve WHERE sa_id = $1 AND cve_id = $2",
                &[&sa_id, &cve_id],
            )
            .await?;

        if let Some(row) = row {
            let sa_cve = self.row_to_sa_cve(&row);
            debug!(
                "成功查询到 SA 与 CVE 关联信息，SA ID: {}, CVE ID: {}",
                sa_id, cve_id
            );
            Ok(Some(sa_cve))
        } else {
            debug!(
                "未找到 SA 与 CVE 关联信息，SA ID: {}, CVE ID: {}",
                sa_id, cve_id
            );
            Ok(None)
        }
    }

    /// 获取特定 SA 的所有 CVE 关联信息
    pub async fn get_sa_cve_by_sa_id(&self, sa_id: i32) -> Result<Vec<SaCve>, DatabaseError> {
        info!("查询 SA 的所有 CVE 关联信息，SA ID: {}", sa_id);

        let rows = self
            .db_manager
            .client
            .query(
                "SELECT sa_id, cve_id FROM sa_cve WHERE sa_id = $1",
                &[&sa_id],
            )
            .await?;

        let sa_cve_list: Vec<SaCve> = rows.iter().map(|row| self.row_to_sa_cve(row)).collect();
        debug!("成功查询到 {} 条 SA 与 CVE 关联信息", sa_cve_list.len());
        Ok(sa_cve_list)
    }

    /// 获取特定 CVE 的所有 SA 关联信息
    pub async fn get_sa_cve_by_cve_id(&self, cve_id: i32) -> Result<Vec<SaCve>, DatabaseError> {
        info!("查询 CVE 的所有 SA 关联信息，CVE ID: {}", cve_id);

        let rows = self
            .db_manager
            .client
            .query(
                "SELECT sa_id, cve_id FROM sa_cve WHERE cve_id = $1",
                &[&cve_id],
            )
            .await?;

        let sa_cve_list: Vec<SaCve> = rows.iter().map(|row| self.row_to_sa_cve(row)).collect();
        debug!("成功查询到 {} 条 SA 与 CVE 关联信息", sa_cve_list.len());
        Ok(sa_cve_list)
    }

    /// 获取所有 SA 与 CVE 关联信息
    pub async fn get_all_sa_cve(&self) -> Result<Vec<SaCve>, DatabaseError> {
        info!("查询所有 SA 与 CVE 关联信息");

        let rows = self
            .db_manager
            .client
            .query(
                "SELECT sa_id, cve_id FROM sa_cve ORDER BY sa_id, cve_id",
                &[],
            )
            .await?;

        let sa_cve_list: Vec<SaCve> = rows.iter().map(|row| self.row_to_sa_cve(row)).collect();
        debug!("成功查询到 {} 条 SA 与 CVE 关联信息", sa_cve_list.len());
        Ok(sa_cve_list)
    }

    /// 根据 ID 获取 CVE 影响信息
    pub async fn get_cve_affect_by_id(&self, id: i32) -> Result<Option<CveAffect>, DatabaseError> {
        info!("查询 CVE 影响信息，ID: {}", id);

        let row = self.db_manager.client.query_opt(
            "SELECT id, cve_id, package_name, os_version_id, status, fixed_version, last_checked FROM cve_affect WHERE id = $1",
            &[&id]
        ).await?;

        if let Some(row) = row {
            let cve_affect = self.row_to_cve_affect(&row);
            debug!("成功查询到 CVE 影响信息，ID: {}", id);
            Ok(Some(cve_affect))
        } else {
            debug!("未找到 CVE 影响信息，ID: {}", id);
            Ok(None)
        }
    }

    /// 根据 CVE ID 获取所有影响信息
    pub async fn get_cve_affects_by_cve_id(
        &self,
        cve_id: i32,
    ) -> Result<Vec<CveAffect>, DatabaseError> {
        info!("查询 CVE 的所有影响信息，CVE ID: {}", cve_id);

        let rows = self.db_manager.client.query(
            "SELECT id, cve_id, package_name, os_version_id, status, fixed_version, last_checked FROM cve_affect WHERE cve_id = $1",
            &[&cve_id]
        ).await?;

        let cve_affect_list: Vec<CveAffect> =
            rows.iter().map(|row| self.row_to_cve_affect(row)).collect();
        debug!("成功查询到 {} 条 CVE 影响信息", cve_affect_list.len());
        Ok(cve_affect_list)
    }

    /// 获取所有 CVE 影响信息
    pub async fn get_all_cve_affects(&self) -> Result<Vec<CveAffect>, DatabaseError> {
        info!("查询所有 CVE 影响信息");

        let rows = self.db_manager.client.query(
            "SELECT id, cve_id, package_name, os_version_id, status, fixed_version, last_checked FROM cve_affect ORDER BY cve_id, id",
            &[]
        ).await?;

        let cve_affect_list: Vec<CveAffect> =
            rows.iter().map(|row| self.row_to_cve_affect(row)).collect();
        debug!("成功查询到 {} 条 CVE 影响信息", cve_affect_list.len());
        Ok(cve_affect_list)
    }

    /// 根据 ID 获取包源码映射信息
    pub async fn get_package_source_map_by_id(
        &self,
        id: i32,
    ) -> Result<Option<PackageSourceMap>, DatabaseError> {
        info!("查询包源码映射信息，ID: {}", id);

        let row = self.db_manager.client.query_opt(
            "SELECT id, package_name, os_version_id, upstream_series, is_inherited, created_at, updated_at FROM package_source_map WHERE id = $1",
            &[&id]
        ).await?;

        if let Some(row) = row {
            let package_source_map = self.row_to_package_source_map(&row);
            debug!("成功查询到包源码映射信息，ID: {}", id);
            Ok(Some(package_source_map))
        } else {
            debug!("未找到包源码映射信息，ID: {}", id);
            Ok(None)
        }
    }

    /// 根据包名获取包源码映射信息
    pub async fn get_package_source_map_by_name(
        &self,
        package_name: &str,
    ) -> Result<Vec<PackageSourceMap>, DatabaseError> {
        info!("查询包源码映射信息，包名: {}", package_name);

        let rows = self.db_manager.client.query(
            "SELECT id, package_name, os_version_id, upstream_series, is_inherited, created_at, updated_at FROM package_source_map WHERE package_name = $1",
            &[&package_name]
        ).await?;

        let package_source_map_list: Vec<PackageSourceMap> = rows
            .iter()
            .map(|row| self.row_to_package_source_map(row))
            .collect();
        debug!(
            "成功查询到 {} 条包源码映射信息",
            package_source_map_list.len()
        );
        Ok(package_source_map_list)
    }

    /// 获取所有包源码映射信息
    pub async fn get_all_package_source_maps(
        &self,
    ) -> Result<Vec<PackageSourceMap>, DatabaseError> {
        info!("查询所有包源码映射信息");

        let rows = self.db_manager.client.query(
            "SELECT id, package_name, os_version_id, upstream_series, is_inherited, created_at, updated_at FROM package_source_map ORDER BY package_name, id",
            &[]
        ).await?;

        let package_source_map_list: Vec<PackageSourceMap> = rows
            .iter()
            .map(|row| self.row_to_package_source_map(row))
            .collect();
        debug!(
            "成功查询到 {} 条包源码映射信息",
            package_source_map_list.len()
        );
        Ok(package_source_map_list)
    }

    /// 根据 ID 获取源码包信息
    pub async fn get_src_rpm_info_by_id(
        &self,
        id: i32,
    ) -> Result<Option<SrcRpmInfo>, DatabaseError> {
        info!("查询源码包信息，ID: {}", id);

        let row = self.db_manager.client.query_opt(
            "SELECT id, package_name, version, release, dist, sa_id, created_at FROM src_rpm_info WHERE id = $1",
            &[&id]
        ).await?;

        if let Some(row) = row {
            let src_rpm_info = self.row_to_src_rpm_info(&row);
            debug!("成功查询到源码包信息，ID: {}", id);
            Ok(Some(src_rpm_info))
        } else {
            debug!("未找到源码包信息，ID: {}", id);
            Ok(None)
        }
    }

    /// 根据包名获取源码包信息
    pub async fn get_src_rpm_info_by_name(
        &self,
        package_name: &str,
    ) -> Result<Vec<SrcRpmInfo>, DatabaseError> {
        info!("查询源码包信息，包名: {}", package_name);

        let rows = self.db_manager.client.query(
            "SELECT id, package_name, version, release, dist, sa_id, created_at FROM src_rpm_info WHERE package_name = $1",
            &[&package_name]
        ).await?;

        let src_rpm_info_list: Vec<SrcRpmInfo> = rows
            .iter()
            .map(|row| self.row_to_src_rpm_info(row))
            .collect();
        debug!("成功查询到 {} 条源码包信息", src_rpm_info_list.len());
        Ok(src_rpm_info_list)
    }

    /// 获取所有源码包信息
    pub async fn get_all_src_rpm_info(&self) -> Result<Vec<SrcRpmInfo>, DatabaseError> {
        info!("查询所有源码包信息");

        let rows = self.db_manager.client.query(
            "SELECT id, package_name, version, release, dist, sa_id, created_at FROM src_rpm_info ORDER BY package_name, id",
            &[]
        ).await?;

        let src_rpm_info_list: Vec<SrcRpmInfo> = rows
            .iter()
            .map(|row| self.row_to_src_rpm_info(row))
            .collect();
        debug!("成功查询到 {} 条源码包信息", src_rpm_info_list.len());
        Ok(src_rpm_info_list)
    }

    /// 根据 ID 获取二进制包信息
    pub async fn get_rpm_info_by_id(&self, id: i32) -> Result<Option<RpmInfo>, DatabaseError> {
        info!("查询二进制包信息，ID: {}", id);

        let row = self.db_manager.client.query_opt(
            "SELECT id, package_name, version, release, dist, arch, src_rpm_id, created_at FROM rpm_info WHERE id = $1",
            &[&id]
        ).await?;

        if let Some(row) = row {
            let rpm_info = self.row_to_rpm_info(&row);
            debug!("成功查询到二进制包信息，ID: {}", id);
            Ok(Some(rpm_info))
        } else {
            debug!("未找到二进制包信息，ID: {}", id);
            Ok(None)
        }
    }

    /// 根据包名获取二进制包信息
    pub async fn get_rpm_info_by_name(
        &self,
        package_name: &str,
    ) -> Result<Vec<RpmInfo>, DatabaseError> {
        info!("查询二进制包信息，包名: {}", package_name);

        let rows = self.db_manager.client.query(
            "SELECT id, package_name, version, release, dist, arch, src_rpm_id, created_at FROM rpm_info WHERE package_name = $1",
            &[&package_name]
        ).await?;

        let rpm_info_list: Vec<RpmInfo> =
            rows.iter().map(|row| self.row_to_rpm_info(row)).collect();
        debug!("成功查询到 {} 条二进制包信息", rpm_info_list.len());
        Ok(rpm_info_list)
    }

    /// 获取所有二进制包信息
    pub async fn get_all_rpm_info(&self) -> Result<Vec<RpmInfo>, DatabaseError> {
        info!("查询所有二进制包信息");

        let rows = self.db_manager.client.query(
            "SELECT id, package_name, version, release, dist, arch, src_rpm_id, created_at FROM rpm_info ORDER BY package_name, id",
            &[]
        ).await?;

        let rpm_info_list: Vec<RpmInfo> =
            rows.iter().map(|row| self.row_to_rpm_info(row)).collect();
        debug!("成功查询到 {} 条二进制包信息", rpm_info_list.len());
        Ok(rpm_info_list)
    }

    /// 根据 ID 获取已处理文件记录
    pub async fn get_processed_file_by_id(
        &self,
        id: i32,
    ) -> Result<Option<ProcessedFile>, DatabaseError> {
        info!("查询已处理文件记录，ID: {}", id);

        let row = self
            .db_manager
            .client
            .query_opt(
                "SELECT id, file_name, file_type, processed_time FROM processed_file WHERE id = $1",
                &[&id],
            )
            .await?;

        if let Some(row) = row {
            let processed_file = self.row_to_processed_file(&row);
            debug!("成功查询到已处理文件记录，ID: {}", id);
            Ok(Some(processed_file))
        } else {
            debug!("未找到已处理文件记录，ID: {}", id);
            Ok(None)
        }
    }

    /// 根据文件名获取已处理文件记录
    pub async fn get_processed_file_by_name(
        &self,
        file_name: &str,
    ) -> Result<Option<ProcessedFile>, DatabaseError> {
        info!("查询已处理文件记录，文件名: {}", file_name);

        let row = self.db_manager.client.query_opt(
            "SELECT id, file_name, file_type, processed_time FROM processed_file WHERE file_name = $1",
            &[&file_name]
        ).await?;

        if let Some(row) = row {
            let processed_file = self.row_to_processed_file(&row);
            debug!("成功查询到已处理文件记录，文件名: {}", file_name);
            Ok(Some(processed_file))
        } else {
            debug!("未找到已处理文件记录，文件名: {}", file_name);
            Ok(None)
        }
    }

    /// 获取所有已处理文件记录
    pub async fn get_all_processed_files(&self) -> Result<Vec<ProcessedFile>, DatabaseError> {
        info!("查询所有已处理文件记录");

        let rows = self.db_manager.client.query(
            "SELECT id, file_name, file_type, processed_time FROM processed_file ORDER BY processed_time DESC",
            &[]
        ).await?;

        let processed_file_list: Vec<ProcessedFile> = rows
            .iter()
            .map(|row| self.row_to_processed_file(row))
            .collect();
        debug!("成功查询到 {} 条已处理文件记录", processed_file_list.len());
        Ok(processed_file_list)
    }

    /// 将数据库行转换为 SaInfo 实体
    fn row_to_sa_info(&self, row: &Row) -> SaInfo {
        SaInfo {
            id: row.get("id"),
            sa_id: row.get("sa_id"),
            synopsis: row.get("synopsis"),
            summary: row.get("summary"),
            topic: row.get("topic"),
            description: row.get("description"),
            severity: row.get("severity"),
            affected_product: row.get("affected_product"),
            affected_component: row.get("affected_component"),
            status: row.get("status"),
            created_time: row.get("created_time"),
            updated_time: row.get("updated_time"),
        }
    }

    /// 将数据库行转换为 CveInfo 实体
    fn row_to_cve_info(&self, row: &Row) -> CveInfo {
        CveInfo {
            id: row.get("id"),
            cve_id: row.get("cve_id"),
            description: row.get("description"),
            base_severity: row.get("base_severity"),
            base_score: row.get("base_score"),
            vector_string: row.get("vector_string"),
            cvss_version: row.get("cvss_version"),
            published_date: row.get("published_date"),
            updated_date: row.get("updated_date"),
            status: row.get("status"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    /// 将数据库行转换为 OsVersionMap 实体
    fn row_to_os_version_map(&self, row: &Row) -> OsVersionMap {
        OsVersionMap {
            id: row.get("id"),
            os_version: row.get("os_version"),
            upstream_series: row.get("upstream_series"),
            dist: row.get("dist"),
            release_date: row.get("release_date"),
            end_of_life: row.get("end_of_life"),
            description: row.get("description"),
        }
    }

    /// 将数据库行转换为 SaCve 实体
    fn row_to_sa_cve(&self, row: &Row) -> SaCve {
        SaCve {
            sa_id: row.get("sa_id"),
            cve_id: row.get("cve_id"),
        }
    }

    /// 将数据库行转换为 CveAffect 实体
    fn row_to_cve_affect(&self, row: &Row) -> CveAffect {
        CveAffect {
            id: row.get("id"),
            cve_id: row.get("cve_id"),
            package_name: row.get("package_name"),
            os_version_id: row.get("os_version_id"),
            status: row.get("status"),
            fixed_version: row.get("fixed_version"),
            last_checked: row.get("last_checked"),
        }
    }

    /// 将数据库行转换为 PackageSourceMap 实体
    fn row_to_package_source_map(&self, row: &Row) -> PackageSourceMap {
        PackageSourceMap {
            id: row.get("id"),
            package_name: row.get("package_name"),
            os_version_id: row.get("os_version_id"),
            upstream_series: row.get("upstream_series"),
            is_inherited: row.get("is_inherited"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    /// 将数据库行转换为 SrcRpmInfo 实体
    fn row_to_src_rpm_info(&self, row: &Row) -> SrcRpmInfo {
        SrcRpmInfo {
            id: row.get("id"),
            package_name: row.get("package_name"),
            version: row.get("version"),
            release: row.get("release"),
            dist: row.get("dist"),
            sa_id: row.get("sa_id"),
            created_at: row.get("created_at"),
        }
    }

    /// 将数据库行转换为 RpmInfo 实体
    fn row_to_rpm_info(&self, row: &Row) -> RpmInfo {
        RpmInfo {
            id: row.get("id"),
            package_name: row.get("package_name"),
            version: row.get("version"),
            release: row.get("release"),
            dist: row.get("dist"),
            arch: row.get("arch"),
            src_rpm_id: row.get("src_rpm_id"),
            created_at: row.get("created_at"),
        }
    }

    /// 将数据库行转换为 ProcessedFile 实体
    fn row_to_processed_file(&self, row: &Row) -> ProcessedFile {
        ProcessedFile {
            id: row.get("id"),
            file_name: row.get("file_name"),
            file_type: row.get("file_type"),
            processed_time: row.get("processed_time"),
        }
    }
}
