//! 数据库表结构模块
//!
//! 该模块提供了数据库表结构初始化相关的功能实现。

use crate::{DatabaseError, DatabaseManager};
use log::info;

impl DatabaseManager {
    /// 清空并重新创建数据库表结构
    pub async fn reinit_tables(&mut self) -> Result<(), DatabaseError> {
        info!("正在清空并重新创建数据库表结构");

        // 删除现有表（按依赖顺序）
        let drop_tables = vec![
            "DROP TABLE IF EXISTS rpminfo_states CASCADE",
            "DROP TABLE IF EXISTS rpminfo_objects CASCADE",
            "DROP TABLE IF EXISTS rpminfo_tests CASCADE",
            "DROP TABLE IF EXISTS cves CASCADE",
            "DROP TABLE IF EXISTS references_info CASCADE",
            "DROP TABLE IF EXISTS oval_definitions CASCADE",
            "DROP TABLE IF EXISTS os_info CASCADE",
        ];

        for drop_query in drop_tables {
            self.client.execute(drop_query, &[]).await?;
        }

        // 重新创建表结构
        self.init_tables().await?;

        info!("数据库表结构重新创建完成");
        Ok(())
    }

    /// 初始化简化版数据库表结构
    pub async fn init_tables(&mut self) -> Result<(), DatabaseError> {
        info!("正在初始化数据库表结构");

        // 创建操作系统信息表（基础表，无外键依赖）
        self.client
            .execute(
                "CREATE TABLE IF NOT EXISTS os_info (
                id BIGSERIAL PRIMARY KEY,
                os_type TEXT NOT NULL,
                os_version TEXT NOT NULL,
                package_name TEXT NOT NULL,
                verify_file TEXT NOT NULL,
                verify_pattern TEXT NOT NULL,
                dist TEXT NOT NULL UNIQUE,
                description TEXT,
                UNIQUE (os_type, os_version)
            )",
                &[],
            )
            .await?;

        // 创建OVAL定义表（包含os_info_id外键）
        self.client
            .execute(
                "CREATE TABLE IF NOT EXISTS oval_definitions (
                id TEXT PRIMARY KEY,
                class TEXT NOT NULL,
                version INTEGER NOT NULL,
                title TEXT NOT NULL,
                description TEXT,
                family TEXT,
                platform TEXT,
                severity TEXT,
                rights TEXT,
                from_field TEXT,
                issued_date TEXT,
                updated_date TEXT,
                os_info_id BIGINT,
                FOREIGN KEY (os_info_id) REFERENCES os_info(id) ON DELETE SET NULL
            )",
                &[],
            )
            .await?;

        // 创建引用信息表（将references重命名为references_info以避免与保留关键字冲突）
        self.client
            .execute(
                "CREATE TABLE IF NOT EXISTS references_info (
                id BIGSERIAL PRIMARY KEY,
                oval_definition_id TEXT NOT NULL,
                ref_id TEXT,
                ref_url TEXT,
                source TEXT,
                FOREIGN KEY (oval_definition_id) REFERENCES oval_definitions(id) ON DELETE CASCADE,
                UNIQUE (oval_definition_id, ref_id)
            )",
                &[],
            )
            .await?;

        // 创建CVE信息表
        self.client
            .execute(
                "CREATE TABLE IF NOT EXISTS cves (
                id BIGSERIAL PRIMARY KEY,
                oval_definition_id TEXT NOT NULL,
                cve_id TEXT,
                cvss3 TEXT,
                impact TEXT,
                href TEXT,
                content TEXT,
                FOREIGN KEY (oval_definition_id) REFERENCES oval_definitions(id) ON DELETE CASCADE,
                UNIQUE (oval_definition_id, cve_id)
            )",
                &[],
            )
            .await?;

        // 创建RPM信息测试表
        self.client
            .execute(
                "CREATE TABLE IF NOT EXISTS rpminfo_tests (
                id BIGSERIAL PRIMARY KEY,
                oval_definition_id TEXT NOT NULL,
                check_field TEXT,
                comment TEXT,
                test_id TEXT,
                version INTEGER,
                object_ref TEXT,
                state_ref TEXT,
                FOREIGN KEY (oval_definition_id) REFERENCES oval_definitions(id) ON DELETE CASCADE,
                UNIQUE (oval_definition_id, test_id)
            )",
                &[],
            )
            .await?;

        // 创建RPM信息对象表
        self.client
            .execute(
                "CREATE TABLE IF NOT EXISTS rpminfo_objects (
                id BIGSERIAL PRIMARY KEY,
                oval_definition_id TEXT NOT NULL,
                object_id TEXT,
                ver BIGINT,
                rpm_name TEXT,
                FOREIGN KEY (oval_definition_id) REFERENCES oval_definitions(id) ON DELETE CASCADE,
                UNIQUE (oval_definition_id, object_id)
            )",
                &[],
            )
            .await?;

        // 创建RPM信息状态表（合并EVR信息）
        self.client
            .execute(
                "CREATE TABLE IF NOT EXISTS rpminfo_states (
                id BIGSERIAL PRIMARY KEY,
                state_id TEXT NOT NULL,
                oval_definition_id TEXT NOT NULL,
                version TEXT,
                evr_datatype TEXT,
                evr_operation TEXT,
                evr_value TEXT,
                FOREIGN KEY (oval_definition_id) REFERENCES oval_definitions(id) ON DELETE CASCADE,
                UNIQUE (oval_definition_id, state_id)
            )",
                &[],
            )
            .await?;

        // 创建ID计数器表，用于持久化ID计数器值
        self.client
            .execute(
                "CREATE TABLE IF NOT EXISTS id_counters (
                id TEXT PRIMARY KEY,
                counter_value BIGINT NOT NULL,
                updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
            )",
                &[],
            )
            .await?;

        info!("数据库表结构初始化完成");
        Ok(())
    }

    /// 初始化操作系统信息数据
    pub async fn init_os_info_data(&mut self) -> Result<(), DatabaseError> {
        info!("正在初始化操作系统信息数据");

        // 定义操作系统信息数据
        let os_info_data = [
            // (os_type, os_version, package_name, verify_file, verify_pattern, dist, description)
            (
                "openEuler",
                "20.03",
                "openeuler-release",
                "/etc/openeuler-release",
                "^openEuler",
                "oe1",
                "openEuler 20.03 LTS",
            ),
            (
                "openEuler",
                "22.03",
                "openeuler-release",
                "/etc/openeuler-release",
                "^openEuler",
                "oe2203",
                "openEuler 22.03 LTS",
            ),
            (
                "openEuler",
                "24.03",
                "openeuler-release",
                "/etc/openeulr-release",
                "^openEuler",
                "oe2403",
                "openEuler 24.03 LTS",
            ),
            (
                "Red Hat Enterprise Linux",
                "7",
                "redhat-release",
                "/etc/redhat-release",
                "^Red Hat Enterprise Linux",
                "el7",
                "Red Hat Enterprise Linux 7",
            ),
            (
                "Red Hat Enterprise Linux",
                "9",
                "redhat-release",
                "/etc/redhat-release",
                "^Red Hat Enterprise Linux",
                "el9",
                "Red Hat Enterprise Linux 9",
            ),
            (
                "Red Hat Enterprise Linux",
                "8",
                "redhat-release",
                "/etc/redhat-release",
                "^Red Hat Enterprise Linux",
                "el8",
                "Red Hat Enterprise Linux 8",
            ),
            (
                "CUOS",
                "4.0",
                "culinux-release",
                "/etc/culinux-release",
                "^CULinux",
                "ule4",
                "culinux 4.0 (Telephone)",
            ),
        ];

        // 批量插入数据
        for (os_type, os_version, package_name, verify_file, verify_pattern, dist, description) in
            os_info_data.iter()
        {
            self.client.execute(
                "INSERT INTO os_info (os_type, os_version, package_name, verify_file, verify_pattern, dist, description)
                 VALUES ($1, $2, $3, $4, $5, $6, $7)
                 ON CONFLICT (dist) DO NOTHING",
                &[
                    os_type,
                    os_version,
                    package_name,
                    verify_file,
                    verify_pattern,
                    dist,
                    description,
                ]
            ).await?;
        }

        info!("操作系统信息数据初始化完成");
        Ok(())
    }
}
