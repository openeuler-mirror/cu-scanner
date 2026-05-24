//! 测试CSAF到OVAL转换（包含OS信息匹配）
use csaf::CSAF;
use parser::csaf_to_oval;
use std::error::Error;
fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    println!("测试CSAF到OVAL转换（包含OS信息匹配）\n");
    // 加载CSAF测试文件
    println!("加载CSAF测试文件...");
    let csaf = CSAF::from_file("../test/csaf/csaf-openeuler-sa-2025-1004.json")?;
    todo!();
}
