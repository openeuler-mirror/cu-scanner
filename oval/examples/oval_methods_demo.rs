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
    let cve1 = CVE::new()
        .with_content("CVE-2024-1234".to_string())
        .with_cvss3("7.5".to_string())
        .with_href("https://nvd.nist.gov/vuln/detail/CVE-2024-1234".to_string())
        .with_impact("High".to_string());

    let cve2 = CVE::new()
        .with_content("CVE-2024-5678".to_string())
        .with_cvss3("9.8".to_string())
        .with_href("https://nvd.nist.gov/vuln/detail/CVE-2024-5678".to_string())
        .with_impact("Critical".to_string());

    metadata.advisory.add_cve(cve1);
    metadata.advisory.add_cve(cve2);
    metadata.advisory.severity = "High".to_string();
    metadata.advisory.issued.date = "2024-01-01".to_string();
    metadata.advisory.updated.date = "2024-01-02".to_string();

    println!("  CVE 数量: {}", metadata.advisory.get_cve_count());
    println!("  CVE 列表: {:?}", metadata.advisory.get_cve_ids());
    println!(
        "  包含 CVE-2024-1234: {}",
        metadata.advisory.contains_cve("CVE-2024-1234")
    );
    println!(
        "  包含 CVE-9999-9999: {}",
        metadata.advisory.contains_cve("CVE-9999-9999")
    );
    println!();

    // 创建条件
    println!("【3. 创建检查条件】");
    let mut criteria = Criteria::new().with_operator("AND".to_string());

    let criterion1 = Criterion::new();
    let criterion2 = Criterion::new();
    criteria.add_criterion(criterion1);
    criteria.add_criterion(criterion2);

    println!("  操作符: {}", criteria.operator);
    println!("  条件数量: {}", criteria.get_criterion_count());
    println!("  子条件数量: {}", criteria.get_sub_criteria_count());
    println!();

    // 创建定义
    todo!();
}
