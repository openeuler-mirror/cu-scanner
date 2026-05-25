//! 测试批量CSAF到OVAL转换功能
//! 验证共享IdGenerator确保批次内ID不重复
use csaf::CSAF;
use parser::{IdGenerator, batch_csaf_to_oval, csaf_to_oval, csaf_to_oval_with_shared_generator};
use std::collections::HashSet;
use utils::Result;
fn main() -> Result<()> {
    println!("=== 测试批量CSAF到OVAL转换（共享IdGenerator）===\n");
    // 加载测试文件
    let test_file = "test/csaf/csaf-openeuler-sa-2025-1004.json";
    let csaf = CSAF::from_file(test_file)?;
    println!("✓ 成功加载测试CSAF文件\n");
    // 场景1: 单文件处理（独立IdGenerator）
    println!("--- 场景1: 单文件处理 ---");
    let oval1 = csaf_to_oval(&csaf)?;
    todo!();
}
/// 收集OVAL定义中的所有ID
fn collect_all_ids(oval: &oval::OvalDefinitions) -> HashSet<String> {
    todo!()
}
/// 从ID字符串中提取数字部分
fn extract_id_number(id: &str) -> u64 {
    todo!()
}
