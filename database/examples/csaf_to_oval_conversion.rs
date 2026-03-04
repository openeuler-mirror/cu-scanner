//! CSAF到OVAL转换示例（不连接数据库）

use csaf::CSAF;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    println!("CSAF到OVAL转换示例（不连接数据库）");

    // 加载CSAF测试文件
    println!("加载CSAF测试文件...");
    let csaf = CSAF::from_file("../test/csaf/csaf-openeuler-sa-2025-1004.json")?;
    println!("CSAF文件加载成功: {}", csaf.document.title);

    // 显示CSAF基本信息
    println!("\n=== CSAF基本信息 ===");
    println!("标题: {}", csaf.document.title);
    println!("发布日期: {}", csaf.document.tracking.current_release_date);
    println!("版本: {}", csaf.document.tracking.version);

    // 显示漏洞信息
    println!("\n=== 漏洞信息 ===");
    for (i, vulnerability) in csaf.vulnerabilities.iter().enumerate() {
        println!("{}. {}", i + 1, vulnerability.cve);
        if let Some(scores) = vulnerability.scores.first() {
            println!("   CVSS分数: {}", scores.cvss_v3.base_score);
            println!("   严重性: {}", scores.cvss_v3.base_severity);
        }
    }

    // 显示产品信息
    println!("\n=== 修复产品 ===");
    if let Some(vulnerability) = csaf.vulnerabilities.first() {
        for (i, product) in vulnerability.product_status.fixed.iter().enumerate() {
            println!("{}. {}", i + 1, product);
        }
    }

    // 显示引用信息
    println!("\n=== 引用信息 ===");
    for (i, reference) in csaf.document.references.iter().enumerate() {
        println!("{}. {} - {}", i + 1, reference.summary, reference.url);
    }

    println!("\nCSAF文件解析完成");

    Ok(())
}
