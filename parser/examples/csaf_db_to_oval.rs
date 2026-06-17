//! CSAF数据库到OVAL转换示例程序
//!
//! 该示例程序演示了如何使用csaf_db_parser从CSAF数据库中提取数据并转换为OVAL格式。

use csaf_database::DatabaseConfig;
use parser::csaf_db_parser::parse_csaf_database_to_oval;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    todo!()
}
