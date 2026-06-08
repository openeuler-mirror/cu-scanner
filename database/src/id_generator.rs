//! 基于数据库的ID生成器模块
//!
//! 该模块提供了基于数据库的ID生成功能，确保ID的全局唯一性和持久化。

use crate::{DatabaseError, DatabaseManager, PersistentIdCounter};
use log::{debug, info};
use oval::{
    CU_LINUX_BA_TST_PREFIX, CU_LINUX_SA_DEF_PREFIX, CU_LINUX_SA_OBJ_PREFIX, CU_LINUX_SA_STE_PREFIX,
    CU_LINUX_SA_TST_PREFIX,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

/// 基于数据库的ID生成器
#[derive(Debug)]
pub struct DatabaseIdGenerator {
    /// 对象ID映射，确保相同对象使用相同ID
    object_ids: HashMap<String, String>,
    /// 状态ID映射，确保相同状态使用相同ID
    state_ids: HashMap<String, String>,
    /// 测试ID映射，确保相同测试使用相同ID
    test_ids: HashMap<String, String>,
    /// 定义ID映射，确保相同定义使用相同ID
    definition_ids: HashMap<String, String>,
    /// ID计数器
    id_counter: PersistentIdCounter,
}

impl DatabaseIdGenerator {
    /// 创建新的基于数据库的ID生成器
    pub fn new(
        db_manager: Arc<Mutex<DatabaseManager>>,
        counter_id: String,
        initial_counter: u64,
    ) -> Self {
        info!(
            "创建新的基于数据库的ID生成器，计数器ID: {}, 初始计数器值: {}",
            counter_id, initial_counter
        );
        let id_counter = PersistentIdCounter::new(db_manager.clone(), counter_id, initial_counter);
        Self {
            object_ids: HashMap::new(),
            state_ids: HashMap::new(),
            test_ids: HashMap::new(),
            definition_ids: HashMap::new(),
            id_counter,
        }
    }

    /// 生成唯一ID
    async fn generate_unique_id(&mut self, prefix: &str) -> Result<String, DatabaseError> {
        let id = self.id_counter.generate_unique_id(prefix).await?;
        debug!("生成唯一ID: {}{}", prefix, id);
        Ok(id)
    }

    /// 获取当前计数器值
    pub async fn get_current_counter(&mut self) -> Result<u64, DatabaseError> {
        self.id_counter.get_current_counter().await
    }

    /// 设置当前计数器值
    pub async fn set_current_counter(&mut self, counter: u64) -> Result<(), DatabaseError> {
        self.id_counter.set_current_counter(counter).await
    }

    /// 获取或创建对象ID，确保相同对象名使用相同ID
    pub async fn get_or_create_object_id(
        &mut self,
        object_name: &str,
        prefix: &str,
    ) -> Result<String, DatabaseError> {
        if let Some(id) = self.object_ids.get(object_name) {
            debug!("使用现有对象ID: {} -> {}", object_name, id);
            Ok(id.clone())
        } else {
            let id = self.generate_unique_id(prefix).await?;
            self.object_ids.insert(object_name.to_string(), id.clone());
            debug!("创建新对象ID: {} -> {}", object_name, id);
            Ok(id)
        }
    }

    /// 获取或创建状态ID，确保相同EVR使用相同ID
    pub async fn get_or_create_state_id(
        &mut self,
        evr: &str,
        prefix: &str,
    ) -> Result<String, DatabaseError> {
        if let Some(id) = self.state_ids.get(evr) {
            debug!("使用现有状态ID: {} -> {}", evr, id);
            Ok(id.clone())
        } else {
            let id = self.generate_unique_id(prefix).await?;
            self.state_ids.insert(evr.to_string(), id.clone());
            debug!("创建新状态ID: {} -> {}", evr, id);
            Ok(id)
        }
    }

    /// 获取或创建测试ID，确保相同测试使用相同ID
    pub async fn get_or_create_test_id(
        &mut self,
        test_key: &str,
        prefix: &str,
    ) -> Result<String, DatabaseError> {
        if let Some(id) = self.test_ids.get(test_key) {
            debug!("使用现有测试ID: {} -> {}", test_key, id);
            Ok(id.clone())
        } else {
            let id = self.generate_unique_id(prefix).await?;
            self.test_ids.insert(test_key.to_string(), id.clone());
            debug!("创建新测试ID: {} -> {}", test_key, id);
            Ok(id)
        }
    }

    /// 获取或创建定义ID，确保相同定义使用相同ID
    pub async fn get_or_create_definition_id(
        &mut self,
        definition_key: &str,
        prefix: &str,
    ) -> Result<String, DatabaseError> {
        if let Some(id) = self.definition_ids.get(definition_key) {
            debug!("使用现有定义ID: {} -> {}", definition_key, id);
            Ok(id.clone())
        } else {
            let id = self.generate_unique_id(prefix).await?;
            self.definition_ids
                .insert(definition_key.to_string(), id.clone());
            debug!("创建新定义ID: {} -> {}", definition_key, id);
            Ok(id)
        }
    }

    /// 为CSAF漏洞生成定义ID
    pub async fn generate_definition_id_for_cve(
        &mut self,
        cve_id: &str,
    ) -> Result<String, DatabaseError> {
        let key = format!("cve:{}", cve_id);
        let id = self
            .get_or_create_definition_id(&key, CU_LINUX_SA_DEF_PREFIX)
            .await?;
        debug!("为CVE生成定义ID: {} -> {}", cve_id, id);
        Ok(id)
    }

    /// 为软件包生成对象ID
    pub async fn generate_object_id_for_package(
        &mut self,
        package_name: &str,
    ) -> Result<String, DatabaseError> {
        let id = self
            .get_or_create_object_id(package_name, CU_LINUX_SA_OBJ_PREFIX)
            .await?;
        debug!("为软件包生成对象ID: {} -> {}", package_name, id);
        Ok(id)
    }

    /// 为EVR生成状态ID
    pub async fn generate_state_id_for_evr(&mut self, evr: &str) -> Result<String, DatabaseError> {
        let id = self
            .get_or_create_state_id(evr, CU_LINUX_SA_STE_PREFIX)
            .await?;
        debug!("为EVR生成状态ID: {} -> {}", evr, id);
        Ok(id)
    }

    /// 为测试生成ID
    pub async fn generate_test_id(
        &mut self,
        package_name: &str,
        evr: &str,
    ) -> Result<String, DatabaseError> {
        let key = format!("{}:{}", package_name, evr);
        let id = self
            .get_or_create_test_id(&key, CU_LINUX_SA_TST_PREFIX)
            .await?;
        debug!("为测试生成ID: {}:{} -> {}", package_name, evr, id);
        Ok(id)
    }

    /// 为基本测试生成ID（用于OS检查等）
    pub async fn generate_base_test_id(
        &mut self,
        test_type: &str,
    ) -> Result<String, DatabaseError> {
        let key = format!("base:{}", test_type);
        let id = self
            .get_or_create_test_id(&key, CU_LINUX_BA_TST_PREFIX)
            .await?;
        debug!("为基本测试生成ID: {} -> {}", test_type, id);
        Ok(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DatabaseConfig, DatabaseManager};
    use std::sync::Arc;
    use tokio::sync::Mutex;

    #[tokio::test]
    async fn test_database_id_generator() -> Result<(), Box<dyn std::error::Error>> {
        // 创建数据库管理器配置（使用测试数据库配置）
        let db_config =
            DatabaseConfig::new("localhost", 5432, "test_db", "test_user", "test_password");

        // 尝试连接数据库
        let db_manager = match DatabaseManager::new(&db_config).await {
            Ok(manager) => Arc::new(Mutex::new(manager)),
            Err(_) => {
                println!("数据库连接失败，跳过测试");
                return Ok(());
            }
        };

        let mut id_generator =
            DatabaseIdGenerator::new(db_manager, "test_generator".to_string(), 10000);

        // 获取当前计数器值
        let current_counter = id_generator.get_current_counter().await?;
        assert_eq!(current_counter, 10000);

        // 生成对象ID
        let obj_id1 = id_generator
            .generate_object_id_for_package("test-package")
            .await?;
        let obj_id2 = id_generator
            .generate_object_id_for_package("test-package")
            .await?;
        assert_eq!(obj_id1, obj_id2); // 应该是相同的ID

        // 生成状态ID
        let state_id1 = id_generator.generate_state_id_for_evr("1.0-1").await?;
        let state_id2 = id_generator.generate_state_id_for_evr("1.0-1").await?;
        assert_eq!(state_id1, state_id2); // 应该是相同的ID

        // 生成测试ID
        let test_id1 = id_generator
            .generate_test_id("test-package", "1.0-1")
            .await?;
        let test_id2 = id_generator
            .generate_test_id("test-package", "1.0-1")
            .await?;
        assert_eq!(test_id1, test_id2); // 应该是相同的ID

        Ok(())
    }
}
