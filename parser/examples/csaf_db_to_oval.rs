//! CSAF数据库到OVAL转换示例程序
//!
//! 该示例程序演示了如何使用csaf_db_parser从CSAF数据库中提取数据并转换为OVAL格式。

use csaf_database::DatabaseConfig;
use parser::csaf_db_parser::parse_csaf_database_to_oval;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("开始从CSAF数据库解析数据到OVAL格式...");

    // 配置数据库连接
    let db_config = DatabaseConfig::new(
        "localhost",     // 数据库主机
        5432,            // 数据库端口
        "csaf_db",       // 数据库名称
        "csaf_user",     // 用户名
        "csaf_password", // 密码
    );

    // 从CSAF数据库解析数据到OVAL格式
    match parse_csaf_database_to_oval(&db_config).await {
        Ok(oval_definitions) => {
            println!(
                "成功解析数据，共 {} 个OVAL定义",
                oval_definitions.definitions.items.len()
            );

            // 显示前几个定义的ID
            for (i, definition) in oval_definitions
                .definitions
                .items
                .iter()
                .take(3)
                .enumerate()
            {
                println!("  {}. {}", i + 1, definition.id);
            }

            // 将OVAL定义转换为XML字符串
            match oval_definitions.to_oval_string() {
                Ok(xml_string) => {
                    println!("\nOVAL XML输出 (前500个字符):");
                    println!("{}", &xml_string[..std::cmp::min(500, xml_string.len())]);

                    // 保存到文件
                    std::fs::write("output_oval.xml", xml_string)?;
                    println!("\nOVAL XML已保存到 output_oval.xml");
                }
                Err(e) => {
                    eprintln!("转换OVAL定义为XML字符串失败: {}", e);
                }
            }
        }
        Err(e) => {
            eprintln!("从CSAF数据库解析数据到OVAL格式失败: {}", e);
            return Err(e);
        }
    }

    println!("\n示例程序执行完成!");
    Ok(())
}
