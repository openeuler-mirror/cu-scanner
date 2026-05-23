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
        todo!();
    }

    /// 初始化操作系统信息数据
    pub async fn init_os_info_data(&mut self) -> Result<(), DatabaseError> {
        todo!()
    }
}
