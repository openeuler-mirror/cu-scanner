//! OVAL 数据结构方法使用示例
//!
//! 该示例程序演示了如何使用 OVAL 数据结构的各种便捷方法。

use oval::*;

fn main() -> Result<()> {
    println!("=== OVAL 数据结构方法使用示例 ===\n");

    // 创建 OVAL 定义
    let mut oval = OvalDefinitions::new();
    // 使用RFC3339格式（符合xs:dateTime要求）
    oval.generator.time_stamp = "2024-01-01T12:00:00Z".to_string();

    println!("【1. 初始状态检查】");
    println!("  定义数量: {}", oval.get_definition_count());
    println!("  测试数量: {}", oval.get_test_count());
    println!("  对象数量: {}", oval.get_object_count());
    println!("  状态数量: {}", oval.get_state_count());
    println!("  是否为空: {}", oval.is_empty());
    println!();
    todo!();
}
