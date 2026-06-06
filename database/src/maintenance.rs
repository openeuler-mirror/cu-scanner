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
        info!("详细检查数据库中的ID");

        // 检查rpminfo_objects表中的所有ID
        println!("检查rpminfo_objects表中的所有ID:");
        let rows = self
            .client
            .query(
                "SELECT id, object_id, oval_definition_id FROM rpminfo_objects ORDER BY id",
                &[],
            )
            .await?;

        for (i, row) in rows.iter().enumerate() {
            let id: String = row.get("id");
            let object_id: String = row.get("object_id");
            let oval_definition_id: String = row.get("oval_definition_id");

            // 检查ID是否以"o,val,:"开头
            if id.starts_with("o,val,:") {
                println!("发现问题ID: {}", id);
            }

            println!("{}. id: '{}'", i + 1, id);
            if id != object_id {
                println!("   object_id: '{}' (不匹配)", object_id);
            }
            println!("   oval_definition_id: '{}'", oval_definition_id);
        }

        println!("\n检查rpminfo_states表中的所有ID:");
        let rows = self
            .client
            .query(
                "SELECT id, state_id, oval_definition_id FROM rpminfo_states ORDER BY id",
                &[],
            )
            .await?;

        for (i, row) in rows.iter().enumerate() {
            let id: i64 = row.get("id");
            let state_id: String = row.get("state_id");
            let oval_definition_id: String = row.get("oval_definition_id");

            // 检查state_id是否以"o,val,:"开头
            if state_id.starts_with("o,val,:") {
                println!("发现问题state_id: {}", state_id);
            }

            println!("{}. id: {}, state_id: '{}'", i + 1, id, state_id);
            println!("   oval_definition_id: '{}'", oval_definition_id);
        }

        Ok(())
    }

    /// 修复数据库中格式错误的ID
    pub async fn fix_database_ids(&mut self) -> Result<(), DatabaseError> {
        info!("修复数据库中格式错误的ID");

        // 检查并修复rpminfo_objects表中的ID
        println!("检查并修复rpminfo_objects表中的ID...");
        let rows = self
            .client
            .query(
                "SELECT id, object_id FROM rpminfo_objects WHERE object_id LIKE 'o,val,%'",
                &[],
            )
            .await?;

        if !rows.is_empty() {
            println!("发现 {} 个格式错误的object_id，正在修复...", rows.len());
            for row in rows {
                let id: i64 = row.get("id");
                let old_object_id: String = row.get("object_id");
                let new_object_id = old_object_id.replacen("o,val,:", "oval:", 1);
                println!("修复object_id: '{}' -> '{}'", old_object_id, new_object_id);

                // 更新rpminfo_objects表
                self.client
                    .execute(
                        "UPDATE rpminfo_objects SET object_id = $1 WHERE id = $2",
                        &[&new_object_id, &id],
                    )
                    .await?;

                // 更新rpminfo_tests表中的object_ref
                self.client
                    .execute(
                        "UPDATE rpminfo_tests SET object_ref = $1 WHERE object_ref = $2",
                        &[&new_object_id, &old_object_id],
                    )
                    .await?;
            }
        } else {
            println!("rpminfo_objects表中没有发现格式错误的object_id");
        }

        // 检查并修复rpminfo_states表中的state_id
        println!("\n检查并修复rpminfo_states表中的state_id...");
        let rows = self
            .client
            .query(
                "SELECT id, state_id FROM rpminfo_states WHERE state_id LIKE 'o,val,%'",
                &[],
            )
            .await?;

        if !rows.is_empty() {
            println!("发现 {} 个格式错误的state_id，正在修复...", rows.len());
            for row in rows {
                let id: i64 = row.get("id");
                let old_state_id: String = row.get("state_id");
                let new_state_id = old_state_id.replacen("o,val,:", "oval:", 1);
                println!("修复state_id: '{}' -> '{}'", old_state_id, new_state_id);

                // 更新rpminfo_states表
                self.client
                    .execute(
                        "UPDATE rpminfo_states SET state_id = $1 WHERE id = $2",
                        &[&new_state_id, &id],
                    )
                    .await?;

                // 更新rpminfo_tests表中的state_ref
                self.client
                    .execute(
                        "UPDATE rpminfo_tests SET state_ref = $1 WHERE state_ref = $2",
                        &[&new_state_id, &old_state_id],
                    )
                    .await?;
            }
        } else {
            println!("rpminfo_states表中没有发现格式错误的state_id");
        }

        println!("\n数据库ID修复完成");
        Ok(())
    }
}
