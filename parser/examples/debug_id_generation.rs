//! 调试ID生成的示例程序

use csaf::CSAF;
use parser::csaf_to_oval_with_counter;

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("调试ID生成示例程序");

    // 加载第一个CSAF文件
    println!("加载第一个CSAF文件...");
    let csaf1 = CSAF::from_file(
        "/home/fatmouse/workspace/cu-scanner/test/csaf/csaf-openeuler-sa-2025-1004.json",
    )?;
    println!("第一个CSAF文件加载成功: {}", csaf1.document.title);

    // 使用计数器10000转换第一个CSAF文件
    println!("转换第一个CSAF文件...");
    let oval1 = csaf_to_oval_with_counter(&csaf1, 10000)?;
    todo!();
}
