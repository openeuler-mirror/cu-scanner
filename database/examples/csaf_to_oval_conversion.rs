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
    todo!();
}
