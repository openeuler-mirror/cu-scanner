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
    let single_ids = collect_all_ids(&oval1);
    println!("  生成ID数量: {}", single_ids.len());
    println!(
        "  示例ID: {:?}\n",
        single_ids.iter().take(3).collect::<Vec<_>>()
    );
    // 场景2: 批量处理（共享IdGenerator）
    println!("--- 场景2: 批量处理3个相同CSAF ---");
    let csaf_list = vec![&csaf, &csaf, &csaf];
    let oval_list = batch_csaf_to_oval(&csaf_list, 10000)?;
    println!("  成功转换 {} 个OVAL定义", oval_list.len());
    // 收集所有ID并检查重复
    let mut all_ids = HashSet::new();
    let mut duplicate_count = 0;
    for (i, oval) in oval_list.iter().enumerate() {
        let ids = collect_all_ids(oval);
        println!("  OVAL[{}] ID数量: {}", i, ids.len());
        for id in ids {
            if !all_ids.insert(id.clone()) {
                duplicate_count += 1;
            }
        }
    }
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
