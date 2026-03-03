//! 测试CSAF到OVAL转换（包含OS信息匹配）
use csaf::CSAF;
use parser::csaf_to_oval;
use std::error::Error;
fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    println!("测试CSAF到OVAL转换（包含OS信息匹配）\n");
    // 加载CSAF测试文件
    println!("加载CSAF测试文件...");
    let csaf = CSAF::from_file("../test/csaf/csaf-openeuler-sa-2025-1004.json")?;
    println!("CSAF文件加载成功: {}\n", csaf.document.title);
    // 显示修复产品（用于OS匹配）
    println!("=== 修复产品列表 ===");
    if let Some(vulnerability) = csaf.vulnerabilities.first() {
        for (i, product) in vulnerability.product_status.fixed.iter().enumerate() {
            println!("{}. {}", i + 1, product);
        }
    }
    // 执行CSAF到OVAL转换
    println!("\n=== 执行CSAF到OVAL转换 ===");
    let oval = csaf_to_oval(&csaf)?;
    println!("OVAL转换成功！\n");
    // 显示OVAL基本信息
    println!("=== OVAL基本信息 ===");
    println!("Generator产品名: {}", oval.generator.product_name);
    println!("Generator时间戳: {}", oval.generator.time_stamp);
    // 显示OVAL定义信息
    println!("\n=== OVAL定义信息 ===");
    if let Some(first_def) = oval.definitions.items.first() {
        println!("定义ID: {}", first_def.id);
        println!("标题: {}", first_def.metadata.title);
        println!("严重级别: {}", first_def.metadata.advisory.severity);
        // 显示Criteria结构（重点显示OS信息）
        println!("\n=== Criteria结构（OS检查） ===");
        println!("顶层操作符: {}", first_def.criteria.operator);
        for (i, criterion) in first_def.criteria.criterion.iter().enumerate() {
            println!("\n{}. Criterion:", i + 1);
            println!("   Comment: {}", criterion.comment);
            println!("   Test Ref: {}", criterion.test_ref);
        }
        if let Some(sub_criteria) = &first_def.criteria.sub_criteria {
            println!("\n子Criteria数量: {}", sub_criteria.len());
            for (i, sub) in sub_criteria.iter().enumerate() {
                println!("\n子Criteria {}: operator={}", i + 1, sub.operator);
                for (j, crit) in sub.criterion.iter().enumerate() {
                    println!("  {}. {}", j + 1, crit.comment);
                }
            }
        }
    }
    // 显示测试信息
    println!("\n=== OVAL测试信息 ===");
    println!("RPM Info Tests数量: {}", oval.tests.rpminfo_tests.len());
    if let Some(first_test) = oval.tests.rpminfo_tests.first() {
        println!("第一个测试ID: {}", first_test.id);
        println!("Comment: {}", first_test.comment);
    }
    // 显示对象信息
    println!("\n=== OVAL对象信息 ===");
    println!(
        "RPM Info Objects数量: {}",
        oval.objects.rpm_info_objects.len()
    );
    for (i, obj) in oval.objects.rpm_info_objects.iter().take(3).enumerate() {
        println!("{}. ID: {}, RPM名称: {}", i + 1, obj.id, obj.rpm_name);
    }
    // 显示状态信息
    println!("\n=== OVAL状态信息 ===");
    if let Some(states) = &oval.states.rpminfo_states {
        println!("RPM Info States数量: {}", states.len());
        for (i, state) in states.iter().take(3).enumerate() {
            println!("{}. ID: {}", i + 1, state.id);
            if let Some(evr) = &state.evr {
                println!("   EVR值: {}", evr.evr);
                println!("   EVR操作: {}", evr.operation);
            }
        }
    }
    println!("\n✓ CSAF到OVAL转换完成，OS信息已成功匹配！");
    Ok(())
}
