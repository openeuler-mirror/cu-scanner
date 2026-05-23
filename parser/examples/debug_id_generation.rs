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
    println!("第一个CSAF文件转换成功");

    if let Some(states1) = &oval1.states.rpminfo_states {
        println!("第一个文件的状态数量: {}", states1.len());
        for (i, state) in states1.iter().enumerate() {
            println!("  {}. ID: {}, Version: {}", i + 1, state.id, state.version);
            if let Some(evr) = &state.evr {
                println!("     EVR: {} {} {}", evr.datatype, evr.operation, evr.evr);
            }
        }
    }

    // 加载第二个CSAF文件
    println!("\n加载第二个CSAF文件...");
    let csaf2 = CSAF::from_file(
        "/home/fatmouse/workspace/cu-scanner/test/csaf/csaf-openeuler-sa-2025-1009.json",
    )?;
    println!("第二个CSAF文件加载成功: {}", csaf2.document.title);

    // 使用计数器10000转换第二个CSAF文件
    println!("转换第二个CSAF文件...");
    let oval2 = csaf_to_oval_with_counter(&csaf2, 10000)?;
    println!("第二个CSAF文件转换成功");

    if let Some(states2) = &oval2.states.rpminfo_states {
        println!("第二个文件的状态数量: {}", states2.len());
        for (i, state) in states2.iter().enumerate() {
            println!("  {}. ID: {}, Version: {}", i + 1, state.id, state.version);
            if let Some(evr) = &state.evr {
                println!("     EVR: {} {} {}", evr.datatype, evr.operation, evr.evr);
            }
        }
    }

    // 检查是否有重复的state ID
    if let (Some(states1), Some(states2)) =
        (&oval1.states.rpminfo_states, &oval2.states.rpminfo_states)
    {
        for state1 in states1 {
            for state2 in states2 {
                if state1.id == state2.id {
                    println!("\n发现重复的state ID: {}", state1.id);
                    println!("  第一个文件中的state ID: {}", state1.id);
                    println!("  第二个文件中的state ID: {}", state2.id);
                }
            }
        }
    }

    Ok(())
}
