//! CSAF 数据库读取示例程序
//!
//! 该示例程序演示了如何使用 csaf_database crate 从数据库读取 CSAF 相关数据。

use csaf_database::{CsafQuery, DatabaseConfig, DatabaseManager};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // 配置数据库连接
    let db_config = DatabaseConfig::new(
        "localhost",     // 数据库主机
        5432,            // 数据库端口
        "csaf_db",       // 数据库名称
        "csaf_user",     // 用户名
        "csaf_password", // 密码
    );

    // 连接数据库
    println!("正在连接数据库...");
    let db_manager = DatabaseManager::new(&db_config).await?;
    println!("数据库连接成功!");

    // 创建 CSAF 查询器
    let csaf_query = CsafQuery::new(db_manager).await?;

    // 示例1: 获取所有安全公告信息
    println!("\n=== 获取所有安全公告信息 ===");
    match csaf_query.get_all_sa_info().await {
        Ok(sa_list) => {
            println!("找到 {} 条安全公告信息", sa_list.len());
            for sa in sa_list {
                println!(
                    "  - ID: {}, SA ID: {}, 标题: {}, 严重性: {}",
                    sa.id,
                    sa.sa_id,
                    sa.topic.as_deref().unwrap_or("N/A"),
                    sa.severity.as_deref().unwrap_or("N/A")
                );
            }
        }
        Err(e) => {
            eprintln!("获取安全公告信息失败: {}", e);
        }
    }

    // 示例2: 根据 ID 获取安全公告信息
    println!("\n=== 根据 ID 获取安全公告信息 ===");
    let sa_id = 1;
    match csaf_query.get_sa_info_by_id(sa_id).await {
        Ok(Some(sa)) => {
            println!("找到安全公告信息:");
            println!("  - ID: {}", sa.id);
            println!("  - SA ID: {}", sa.sa_id);
            println!("  - 标题: {}", sa.topic.as_deref().unwrap_or("N/A"));
            println!("  - 严重性: {}", sa.severity.as_deref().unwrap_or("N/A"));
        }
        Ok(None) => {
            println!("未找到 ID 为 {} 的安全公告信息", sa_id);
        }
        Err(e) => {
            eprintln!("查询安全公告信息失败: {}", e);
        }
    }

    // 示例3: 根据 SA ID 获取安全公告信息
    println!("\n=== 根据 SA ID 获取安全公告信息 ===");
    let sa_identifier = "SA-2025-1004";
    match csaf_query.get_sa_info_by_sa_id(sa_identifier).await {
        Ok(Some(sa)) => {
            println!("找到安全公告信息:");
            println!("  - ID: {}", sa.id);
            println!("  - SA ID: {}", sa.sa_id);
            println!("  - 标题: {}", sa.topic.as_deref().unwrap_or("N/A"));
        }
        Ok(None) => {
            println!("未找到 SA ID 为 {} 的安全公告信息", sa_identifier);
        }
        Err(e) => {
            eprintln!("查询安全公告信息失败: {}", e);
        }
    }

    // 示例4: 获取所有 CVE 信息
    println!("\n=== 获取所有 CVE 信息 ===");
    match csaf_query.get_all_cve_info().await {
        Ok(cve_list) => {
            println!("找到 {} 条 CVE 信息", cve_list.len());
            for cve in cve_list {
                println!(
                    "  - ID: {}, CVE ID: {}, 描述: {}, 严重性: {}",
                    cve.id,
                    cve.cve_id,
                    &cve.description[..std::cmp::min(50, cve.description.len())],
                    cve.base_severity.as_deref().unwrap_or("N/A")
                );
            }
        }
        Err(e) => {
            eprintln!("获取 CVE 信息失败: {}", e);
        }
    }

    // 示例5: 根据 CVE ID 获取 CVE 信息
    println!("\n=== 根据 CVE ID 获取 CVE 信息 ===");
    let cve_id = "CVE-2025-1004";
    match csaf_query.get_cve_info_by_cve_id(cve_id).await {
        Ok(Some(cve)) => {
            println!("找到 CVE 信息:");
            println!("  - ID: {}", cve.id);
            println!("  - CVE ID: {}", cve.cve_id);
            println!("  - 描述: {}", cve.description);
            println!(
                "  - 严重性: {}",
                cve.base_severity.as_deref().unwrap_or("N/A")
            );
            println!("  - 分数: {}", cve.base_score.unwrap_or(0.0));
        }
        Ok(None) => {
            println!("未找到 CVE ID 为 {} 的 CVE 信息", cve_id);
        }
        Err(e) => {
            eprintln!("查询 CVE 信息失败: {}", e);
        }
    }

    // 示例6: 获取 SA 与 CVE 的关联信息
    println!("\n=== 获取 SA 与 CVE 的关联信息 ===");
    match csaf_query.get_all_sa_cve().await {
        Ok(sa_cve_list) => {
            println!("找到 {} 条 SA 与 CVE 关联信息", sa_cve_list.len());
            for sa_cve in sa_cve_list.iter().take(5) {
                // 只显示前5条
                println!("  - SA ID: {}, CVE ID: {}", sa_cve.sa_id, sa_cve.cve_id);
            }
        }
        Err(e) => {
            eprintln!("获取 SA 与 CVE 关联信息失败: {}", e);
        }
    }

    // 示例7: 获取指定时间之后的安全公告ID列表
    println!("\n=== 获取指定时间之后的安全公告ID列表 ===");
    let timestamp = "2025-01-01 00:00:00";
    match csaf_query.get_sa_ids_after_time(timestamp).await {
        Ok(sa_ids) => {
            println!("在 {} 之后创建的 {} 个安全公告:", timestamp, sa_ids.len());
            for sa_id in sa_ids.iter().take(10) {
                // 只显示前10个
                println!("  - SA ID: {}", sa_id);
            }
            if sa_ids.len() > 10 {
                println!("  ... 还有 {} 个安全公告", sa_ids.len() - 10);
            }
        }
        Err(e) => {
            eprintln!("获取安全公告ID列表失败: {}", e);
        }
    }

    println!("\n示例程序执行完成!");
    Ok(())
}
