//! 测试CSAF文件上传功能的示例程序

use actix_server::{ServerConfig, create_default_server};
use database::DatabaseConfig;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // 创建服务器配置
    let server_config = ServerConfig {
        database_config: DatabaseConfig::new("localhost", 5432, "oval_db", "username", "password"),
        address: "0.0.0.0".to_string(),
        port: 8091,
        api_group_name: "api".to_string(),
    };

    println!("启动测试服务器...");

    // 启动网络服务
    create_default_server(server_config).await?;

    Ok(())
}
