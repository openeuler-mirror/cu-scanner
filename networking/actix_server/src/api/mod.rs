//! API模块的入口文件

pub mod file_handler;
pub mod oval_handler;
pub mod export_handler;

use actix_web::web;

/// 配置API路由，使用指定的API组名
///
/// # 参数
///
/// * `api_group_name` - API组名，将作为路由前缀
///
/// # 返回值
///
/// 返回一个配置闭包，可用于App的configure方法
pub fn get_api_scope(api_group_name: String) -> actix_web::Scope {
    web::scope(&format!("/{}", api_group_name))
        .service(web::scope("/oval")
            .configure(oval_handler::config)
            .service(web::scope("/export").configure(export_handler::config)))
        .configure(file_handler::config)
}
