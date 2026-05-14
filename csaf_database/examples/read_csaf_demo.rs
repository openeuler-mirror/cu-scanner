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
    todo!();
}
