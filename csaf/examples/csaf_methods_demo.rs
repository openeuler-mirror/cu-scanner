//! CSAF 数据结构方法使用示例
//!
//! 该示例程序演示了如何使用 CSAF 数据结构的各种便捷方法。

use csaf::CSAF;
use utils::Result;

fn main() -> Result<()> {
    println!("=== CSAF 数据结构方法使用示例 ===\n");

    // 从测试文件加载 CSAF 数据
    let test_file = "test/csaf/csaf-openeuler-sa-2025-1004.json";
    println!("正在从文件加载 CSAF 数据: {}\n", test_file);

    let csaf = CSAF::from_file(test_file)?;

    // 使用 CSAF 的便捷方法
    println!("【CSAF 文档信息】");
    println!("  文档 ID: {}", csaf.get_id());
    println!("  版本: {}", csaf.get_version());
    println!("  标题: {}", csaf.get_title());
    println!("  状态: {}", csaf.get_status());
    println!("  初始发布日期: {}", csaf.get_initial_release_date());
    println!("  当前发布日期: {}", csaf.get_release_date());
    println!();

    // 文档属性
    println!("【文档属性】");
    println!("  类别: {}", csaf.document.get_category());
    println!("  语言: {}", csaf.document.get_lang());
    println!("  发布者: {}", csaf.document.get_publisher_name());
    println!();

    // 严重性信息
    println!("【严重性信息】");
    println!(
        "  级别: {}",
        csaf.document.aggregate_severity.get_severity()
    );
    println!(
        "  是否严重: {}",
        csaf.document.aggregate_severity.is_critical()
    );
    println!("  是否高危: {}", csaf.document.aggregate_severity.is_high());
    println!();

    // TLP 信息
    println!("【TLP 信息】");
    println!("  标签: {}", csaf.document.distribution.tlp.label);
    println!(
        "  是否可公开共享: {}",
        csaf.document.distribution.tlp.is_public()
    );
    println!();

    // 跟踪信息
    println!("【跟踪信息】");
    println!(
        "  修订历史数量: {}",
        csaf.document.tracking.get_revision_count()
    );
    if let Some(latest) = csaf.document.tracking.get_latest_revision() {
        println!("  最新修订:");
        println!("    日期: {}", latest.date);
        println!("    版本: {}", latest.number);
        println!("    说明: {}", latest.summary);
    }
    todo!();
}
