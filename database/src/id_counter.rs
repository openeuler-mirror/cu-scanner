//! 持久化ID计数器模块
//!
//! 该模块提供了基于数据库的ID计数器功能，确保ID的全局唯一性和持久化。

use crate::{DatabaseError, DatabaseManager};
use log::{debug, info};
use std::sync::Arc;
use tokio::sync::Mutex;

/// 持久化ID计数器
#[derive(Debug)]
pub struct PersistentIdCounter {
    /// 数据库管理器
    db_manager: Arc<Mutex<DatabaseManager>>,
    /// 计数器ID（用于在数据库中标识这个计数器）
    counter_id: String,
    /// 当前计数器值（缓存）
    current_counter: u64,
    /// 是否已从数据库加载
    loaded: bool,
}

impl PersistentIdCounter {
    /// 创建新的持久化ID计数器
    pub fn new(
        db_manager: Arc<Mutex<DatabaseManager>>,
        counter_id: String,
        initial_counter: u64,
    ) -> Self {
        debug!(
            "创建新的持久化ID计数器: {}, 初始计数器值: {}",
            counter_id, initial_counter
        );
        Self {
            db_manager,
            counter_id,
            current_counter: initial_counter,
            loaded: false,
        }
    }

    /// 从数据库加载计数器值
    async fn load_from_database(&mut self) -> Result<(), DatabaseError> {
        if !self.loaded {
            let db_manager = self.db_manager.lock().await;
            if let Some(counter_value) = db_manager.get_id_counter(&self.counter_id).await? {
                self.current_counter = counter_value;
                info!(
                    "从数据库加载计数器值: {} -> {}",
                    self.counter_id, self.current_counter
                );
            }
            self.loaded = true;
        }
        Ok(())
    }

    /// 将计数器值保存到数据库
    async fn save_to_database(&self) -> Result<(), DatabaseError> {
        let db_manager = self.db_manager.lock().await;
        db_manager
            .set_id_counter(&self.counter_id, self.current_counter)
            .await?;
        debug!(
            "将计数器值保存到数据库: {} -> {}",
            self.counter_id, self.current_counter
        );
        Ok(())
    }

    /// 获取当前计数器值
    pub async fn get_current_counter(&mut self) -> Result<u64, DatabaseError> {
        self.load_from_database().await?;
        debug!(
            "获取当前计数器值: {} -> {}",
            self.counter_id, self.current_counter
        );
        Ok(self.current_counter)
    }

    /// 设置当前计数器值
    pub async fn set_current_counter(&mut self, counter: u64) -> Result<(), DatabaseError> {
        self.load_from_database().await?;
        debug!(
            "设置当前计数器值: {} -> {} -> {}",
            self.counter_id, self.current_counter, counter
        );
        self.current_counter = counter;
        self.save_to_database().await?;
        Ok(())
    }

    /// 生成唯一ID
    pub async fn generate_unique_id(&mut self, prefix: &str) -> Result<String, DatabaseError> {
        self.load_from_database().await?;
        self.current_counter += 1;
        // 在数字ID前添加1，避免ID以0开头
        let id = format!("{}1{}", prefix, self.current_counter);
        self.save_to_database().await?;
        debug!("生成唯一ID: {}", id);
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
    async fn test_persistent_id_counter() -> Result<(), Box<dyn std::error::Error>> {
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

        let mut id_counter =
            PersistentIdCounter::new(db_manager, "test_counter".to_string(), 10000);

        // 获取当前计数器值
        let current_counter = id_counter.get_current_counter().await?;
        assert_eq!(current_counter, 10000);

        // 生成唯一ID（现在ID以1开头）
        let id1 = id_counter.generate_unique_id("test:").await?;
        assert_eq!(id1, "test:110001");

        let id2 = id_counter.generate_unique_id("test:").await?;
        assert_eq!(id2, "test:110002");

        // 更新计数器值
        id_counter.set_current_counter(50000).await?;
        let updated_counter = id_counter.get_current_counter().await?;
        assert_eq!(updated_counter, 50000);

        Ok(())
    }
}
