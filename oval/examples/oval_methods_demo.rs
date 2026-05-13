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

    // 创建元数据
    println!("【2. 创建元数据】");
    let mut metadata = Metadata::new();
    metadata.title = "nginx 安全更新".to_string();
    metadata.description = "修复 nginx 中的安全漏洞".to_string();
    metadata.affected.platform = "China Unicom Linux 4".to_string();

    // 添加引用
    let reference = Reference::new();
    metadata.add_reference(reference);
    println!("  标题: {}", metadata.title);
    println!("  引用数量: {}", metadata.get_reference_count());

    // 添加 CVE
    todo!();
}
