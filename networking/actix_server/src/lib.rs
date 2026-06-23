//! Actix Web服务器库
//!
//! 该库提供了基于Actix Web框架的服务器功能。

pub mod api;

use actix_web::{App, HttpServer, web};
use database::DatabaseConfig;
use log::info;

/// Actix服务器配置
pub struct ServerConfig {
    /// 数据库配置
    pub database_config: DatabaseConfig,
    /// 服务器地址，默认为0.0.0.0
    pub address: String,
    /// 服务器端口
    pub port: u16,
    /// API接口组名，默认为"api"
    pub api_group_name: String,
}

/// 启动HTTP服务器
///
/// 该函数启动HTTP服务器并监听指定地址的请求。
///
/// # 参数
///
/// * `ip` - 服务器监听的IP地址
/// * `port` - 服务器监听的端口号
/// * `server_config` - 服务器配置
///
/// # 返回值
///
/// 返回std::io::Result<()>，表示服务器启动是否成功
pub async fn start_server(ip: &str, port: u16, server_config: ServerConfig) -> std::io::Result<()> {
    let address = format!("{}:{}", ip, port);
    info!("正在启动Actix Web服务器，监听地址: {}", address);

    let api_group_name = server_config.api_group_name.clone();
    let db_config = server_config.database_config.clone();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(db_config.clone()))
            .service(api::get_api_scope(api_group_name.clone()))
    })
    .bind(address)?
    .run()
    .await
}

/// 创建默认的HTTP服务器
///
/// 该函数使用配置中指定的地址和端口创建HTTP服务器。
///
/// # 参数
///
/// * `server_config` - 服务器配置
///
/// # 返回值
///
/// 返回std::io::Result<()>，表示服务器创建是否成功
pub async fn create_default_server(server_config: ServerConfig) -> std::io::Result<()> {
    let address = server_config.address.clone();
    let port = server_config.port;
    start_server(&address, port, server_config).await
}
