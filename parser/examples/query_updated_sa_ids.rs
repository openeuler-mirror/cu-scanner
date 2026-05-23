//! 查询更新时间之后的SA ID列表示例程序
//!
//! 该示例程序演示了如何从CSAF数据库中查询某个时间之后更新的所有SA ID。

use csaf_database::DatabaseConfig;
use parser::csaf_db_parser::get_sa_ids_after_updated_time;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("=== 查询更新时间之后的SA ID列表示例 ===\n");

    // 配置 CSAF 数据库连接
    // 注意：实际使用时应从配置文件读取或使用环境变量
    let csaf_db_config = DatabaseConfig::new(
        "82.156.53.132", // 数据库主机
        5432,            // 数据库端口
        "cu_cveadmin",   // 数据库名称
        "cu_cveadmin",   // 用户名
        "Ninjia1000SX",  // 密码
    );

    println!("CSAF 数据库配置:");
    println!("  主机: {}", csaf_db_config.host);
    println!("  端口: {}", csaf_db_config.port);
    println!("  数据库: {}", csaf_db_config.database);
    println!("  用户名: {}", csaf_db_config.username);
    println!();

    // 设置查询的时间点
    // 可以使用以下格式：
    // - "2025-01-01"
    // - "2025-01-01 00:00:00"
    // - "2025-10-01T00:00:00"
    let timestamp = "2025-10-01";
    println!("查询更新时间在 {} 之后的所有 SA ID...\n", timestamp);

    // 查询更新时间之后的 SA ID
    match get_sa_ids_after_updated_time(&csaf_db_config, timestamp).await {
        Ok(sa_ids) => {
            println!("成功查询到 {} 个 SA ID:\n", sa_ids.len());

            // 显示所有 SA ID
            if sa_ids.is_empty() {
                println!("  (没有找到符合条件的 SA ID)");
            } else {
                for (i, sa_id) in sa_ids.iter().enumerate() {
                    println!("  {}. {}", i + 1, sa_id);
                }
            }

            println!("\n查询成功！");
        }
        Err(e) => {
            eprintln!("查询 SA ID 失败: {}", e);
            return Err(e);
        }
    }

    Ok(())
}
