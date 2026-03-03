//! ID计数器管理模块
//!
//! 该模块提供了生成唯一ID的功能，确保ID的唯一性和引用一致性。

use log::debug;

/// ID计数器管理器
#[derive(Debug)]
pub struct IdCounterManager {
    current_counter: u64,
}

impl IdCounterManager {
    /// 创建新的ID计数器管理器
    pub fn new(initial_counter: u64) -> Self {
        debug!("创建新的ID计数器管理器，初始计数器值: {}", initial_counter);
        Self {
            current_counter: initial_counter,
        }
    }

    /// 获取当前计数器值
    pub fn get_current_counter(&self) -> u64 {
        debug!("获取当前计数器值: {}", self.current_counter);
        self.current_counter
    }

    /// 设置当前计数器值
    pub fn set_current_counter(&mut self, counter: u64) {
        debug!("设置当前计数器值: {} -> {}", self.current_counter, counter);
        self.current_counter = counter;
    }

    /// 生成唯一ID
    pub fn generate_unique_id(&mut self, prefix: &str) -> String {
        self.current_counter += 1;
        // 在数字ID前添加1，避免ID以0开头
        let id = format!("{}1{}", prefix, self.current_counter);
        debug!("生成唯一ID: {}", id);
        id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_id_counter_operations() {
        let mut id_counter = IdCounterManager::new(10000);

        // 获取当前计数器值
        let current_counter = id_counter.get_current_counter();
        assert_eq!(current_counter, 10000);

        // 生成唯一ID（现在ID以1开头）
        let id1 = id_counter.generate_unique_id("test:");
        assert_eq!(id1, "test:110001");

        let id2 = id_counter.generate_unique_id("test:");
        assert_eq!(id2, "test:110002");

        // 更新计数器值
        id_counter.set_current_counter(50000);
        let updated_counter = id_counter.get_current_counter();
        assert_eq!(updated_counter, 50000);
    }
}
