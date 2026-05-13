//! 根据 SA ID 从数据库创建 OVAL 定义示例程序
//!
//! 该示例程序演示了如何根据 SA ID 从 CSAF 数据库中查询信息并创建 OVAL 定义。

use csaf_database::DatabaseConfig;
use parser::csaf_db_parser::create_definition_from_sa_id;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("=== 根据 SA ID 创建 OVAL 定义示例 ===\n");

    // 配置 CSAF 数据库连接
    let csaf_db_config = DatabaseConfig::new(
        "82.156.53.132", // 数据库主机
        5432,            // 数据库端口
        "cu_cveadmin",   // 数据库名称
        "cu_cveadmin",   // 用户名
        "Ninjia1000SX",  // 密码
    );

    println!("CSAF 数据库配置:");
    println!("  主机: {}", csaf_db_config.host);
    todo!();
}
