//! 数据库维护模块
//!
//! 该模块提供了数据库检查和修复相关的功能实现。

use crate::{DatabaseError, DatabaseManager};
use log::info;

impl DatabaseManager {
    /// 检查表结构
    pub async fn check_table_structure(&self, table_name: &str) -> Result<(), DatabaseError> {
        println!("检查{}表结构...", table_name);
        let rows = self.client.query(
            "SELECT column_name, data_type FROM information_schema.columns WHERE table_name = $1 ORDER BY ordinal_position",
            &[&table_name]
        ).await?;

        println!("{}表列信息:", table_name);
        for row in rows {
            let column_name: String = row.get(0);
            let data_type: String = row.get(1);
            println!("  - {}: {}", column_name, data_type);
        }

        Ok(())
    }

    /// 检查rpminfo_objects表中的数据
    pub async fn check_rpminfo_objects(&self) -> Result<(), DatabaseError> {
        info!("检查rpminfo_objects表中的数据");
        let rows = self
            .client
            .query(
                "SELECT id, object_id, oval_definition_id FROM rpminfo_objects LIMIT 10",
                &[],
            )
            .await?;

        println!("rpminfo_objects表中的数据:");
        for (i, row) in rows.iter().enumerate() {
            let id: i64 = row.get("id");
            let object_id: String = row.get("object_id");
            let oval_definition_id: String = row.get("oval_definition_id");
            println!(
                "{}. id: {}, object_id: '{}', oval_definition_id: '{}'",
                i + 1,
                id,
                object_id,
                oval_definition_id
            );
        }

        Ok(())
    }

    /// 检查rpminfo_states表中的数据（包含EVR信息）
    pub async fn check_rpminfo_states(&self) -> Result<(), DatabaseError> {
        info!("检查rpminfo_states表中的数据");
        let rows = self.client.query(
            "SELECT id, state_id, oval_definition_id, evr_datatype, evr_operation, evr_value FROM rpminfo_states LIMIT 10",
            &[]
        ).await?;

        println!("rpminfo_states表中的数据（包含EVR信息）:");
        for (i, row) in rows.iter().enumerate() {
            let id: i64 = row.get("id");
            let state_id: String = row.get("state_id");
            let oval_definition_id: String = row.get("oval_definition_id");
            let evr_datatype: Option<String> = row.get("evr_datatype");
            let evr_operation: Option<String> = row.get("evr_operation");
            let evr_value: Option<String> = row.get("evr_value");
            println!(
                "{}. id: {}, state_id: '{}', oval_definition_id: '{}'",
                i + 1,
                id,
                state_id,
                oval_definition_id
            );
            if let (Some(dt), Some(op), Some(val)) = (evr_datatype, evr_operation, evr_value) {
                println!("   EVR: {} {} {}", dt, op, val);
            }
        }

        Ok(())
    }

    /// 清空所有数据表
    pub async fn clear_all_tables(&mut self) -> Result<(), DatabaseError> {
        info!("清空所有数据表");

        // 按依赖顺序删除数据
        let clear_queries = vec![
            "DELETE FROM rpminfo_states",
            "DELETE FROM rpminfo_objects",
            "DELETE FROM rpminfo_tests",
            "DELETE FROM cves",
            "DELETE FROM references_info",
            "DELETE FROM oval_definitions",
        ];

        for query in clear_queries {
            self.client.execute(query, &[]).await?;
        }

        info!("所有数据表已清空");
        Ok(())
    }

    /// 详细检查数据库中的ID
    pub async fn check_database_ids(&self) -> Result<(), DatabaseError> {
        todo!()
    }

    /// 修复数据库中格式错误的ID
    pub async fn fix_database_ids(&mut self) -> Result<(), DatabaseError> {
        todo!()
    }
}
