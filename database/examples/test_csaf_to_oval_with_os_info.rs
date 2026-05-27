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
    todo!();
}
