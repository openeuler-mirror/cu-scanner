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
    println!();

    // 产品信息
    println!("【产品信息】");
    println!("  产品数量: {}", csaf.product_tree.get_product_count());
    let product_ids = csaf.product_tree.get_all_product_ids();
    println!("  产品列表 (前3个):");
    for (i, pid) in product_ids.iter().take(3).enumerate() {
        println!("    {}. {}", i + 1, pid);
    }
    println!();

    // 漏洞信息
    println!("【漏洞信息】");
    println!("  漏洞数量: {}", csaf.get_vulnerability_count());

    let cve_ids = csaf.get_cve_ids();
    println!("  CVE 列表:");
    for (i, cve_id) in cve_ids.iter().enumerate() {
        println!("    {}. {}", i + 1, cve_id);
    }
    println!();

    // 详细漏洞信息
    if let Some(vuln) = csaf.vulnerabilities.first() {
        println!("【第一个漏洞详情】");
        println!("  CVE ID: {}", vuln.get_cve_id());
        println!("  标题: {}", vuln.get_title());
        println!("  受影响产品数量: {}", vuln.get_affected_product_count());

        if let Some(score) = vuln.get_cvss_score() {
            println!("  CVSS 分数: {}", score);
        }

        if let Some(severity) = vuln.get_severity() {
            println!("  严重性: {}", severity);
        }

        println!("  是否严重漏洞: {}", vuln.is_critical());
        println!("  是否高危漏洞: {}", vuln.is_high());
        println!();

        // CVSS 详细信息
        if let Some(score) = vuln.scores.first() {
            println!("【CVSS 详细信息】");
            println!("  基础分数: {}", score.get_base_score());
            println!("  严重性: {}", score.get_severity());
            println!("  向量字符串: {}", score.get_vector_string());
            println!("  是否严重: {}", score.cvss_v3.is_critical());
            println!("  是否高危: {}", score.cvss_v3.is_high());
            println!("  是否中危: {}", score.cvss_v3.is_medium());
            println!("  是否低危: {}", score.cvss_v3.is_low());
            println!();
        }

        // 产品状态
        println!("【产品状态】");
        let fixed_products = vuln.product_status.get_fixed_products();
        println!("  已修复产品数量: {}", fixed_products.len());
        if let Some(first_product) = fixed_products.first() {
            println!("  示例检查产品是否已修复:");
            println!("    产品: {}", first_product);
            println!(
                "    已修复: {}",
                vuln.product_status.is_product_fixed(first_product)
            );
        }
    }

    // 检查特定 CVE
    println!("\n【CVE 检查】");
    if let Some(first_cve) = cve_ids.first() {
        println!(
            "  检查是否包含 {}: {}",
            first_cve,
            csaf.contains_cve(first_cve)
        );
    }
    println!(
        "  检查是否包含 CVE-9999-9999: {}",
        csaf.contains_cve("CVE-9999-9999")
    );

    println!("\n示例执行完成！");
    Ok(())
}
