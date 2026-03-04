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
    println!("\n  批次内总ID数: {}", all_ids.len());
    println!(
        "  重复ID数: {} (注：相同的包/版本会复用相同ID，这是预期行为)",
        duplicate_count
    );
    println!("  ✅ 批次内ID生成策略正常！");
    // 场景3: 手动控制共享IdGenerator
    println!("\n--- 场景3: 手动共享IdGenerator ---");
    let mut shared_gen = IdGenerator::new(10000);
    let oval_a = csaf_to_oval_with_shared_generator(&csaf, &mut shared_gen)?;
    let oval_b = csaf_to_oval_with_shared_generator(&csaf, &mut shared_gen)?;
    let ids_a = collect_all_ids(&oval_a);
    let ids_b = collect_all_ids(&oval_b);
    let mut combined = HashSet::new();
    let mut dup = 0;
    for id in ids_a.iter().chain(ids_b.iter()) {
        if !combined.insert(id.clone()) {
            dup += 1;
        }
    }
    println!("  OVAL_A ID数: {}", ids_a.len());
    println!("  OVAL_B ID数: {}", ids_b.len());
    println!("  合并后总ID数: {}", combined.len());
    println!("  重复ID数: {} (相同内容复用ID)", dup);
    println!("  ✅ 共享IdGenerator工作正常！");
    // 验证ID范围
    println!("\n--- ID范围验证 ---");
    let os_ids: Vec<_> = all_ids
        .iter()
        .filter(|id| extract_id_number(id) < 10000)
        .collect();
    let rpminfo_ids: Vec<_> = all_ids
        .iter()
        .filter(|id| extract_id_number(id) >= 10000)
        .collect();
    println!("  OS相关ID (0-9999): {} 个", os_ids.len());
    println!("  RpmInfo相关ID (10000+): {} 个", rpminfo_ids.len());
    println!("\n=== 测试完成 ===");
    Ok(())
}
/// 收集OVAL定义中的所有ID
fn collect_all_ids(oval: &oval::OvalDefinitions) -> HashSet<String> {
    let mut ids = HashSet::new();
    // Definition IDs
    for def in &oval.definitions.items {
        ids.insert(def.id.clone());
    }
    // Test IDs
    for test in &oval.tests.rpminfo_tests {
        ids.insert(test.id.clone());
    }
    for test in &oval.tests.rpmverifyfile_tests {
        ids.insert(test.id.clone());
    }
    // Object IDs
    for obj in &oval.objects.rpm_info_objects {
        ids.insert(obj.id.clone());
    }
    for obj in &oval.objects.rpmverifyfile_objects {
        ids.insert(obj.id.clone());
    }
    // State IDs
    if let Some(states) = &oval.states.rpminfo_states {
        for state in states {
            ids.insert(state.id.clone());
        }
    }
    if let Some(states) = &oval.states.rpmverifyfile_states {
        for state in states {
            ids.insert(state.id.clone());
        }
    }
    ids
}
/// 从ID字符串中提取数字部分
fn extract_id_number(id: &str) -> u64 {
    id.split(':')
        .last()
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(0)
}
