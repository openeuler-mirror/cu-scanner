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
    todo!();
}
