//! 文件处理相关的API接口

use actix_web::{HttpResponse, Responder, post, web};
use csaf::CSAF;
use database::{DatabaseConfig, DatabaseManager};
use log::{error, info};
use parser::csaf_to_oval;
use serde::{Deserialize, Serialize};

/// 上传响应结构
#[derive(Serialize, Deserialize)]
pub struct UploadResponse {
    /// 是否成功
    pub success: bool,
    /// 消息
    pub message: String,
    /// OVAL定义ID（仅在成功时提供）
    pub oval_id: Option<String>,
}

/// 上传CSAF文件
///
/// # 参数
///
/// * `file_name` - 文件名
/// * `body` - 请求体内容（CSAF文件内容）
/// * `db_config` - 数据库配置
///
/// # 返回值
///
/// 返回JSON格式的响应，包含成功或失败信息
#[post("/file/{file_name}")]
pub async fn post_csaf_file(
    file_name: web::Path<String>,
    body: String,
    db_config: web::Data<DatabaseConfig>,
) -> impl Responder {
    info!("收到上传CSAF文件请求，文件名: {}", file_name);

    // 解析CSAF内容
    let csaf = match serde_json::from_str::<CSAF>(&body) {
        Ok(csaf) => {
            info!("CSAF文件解析成功");
            csaf
        }
        Err(e) => {
            error!("CSAF文件解析失败: {}", e);
            let response = UploadResponse {
                success: false,
                message: format!("CSAF文件解析失败: {}", e),
                oval_id: None,
            };
            return HttpResponse::BadRequest().json(response);
        }
    };
    todo!();
}

/// 配置文件处理相关的路由
pub fn config(cfg: &mut web::ServiceConfig) {
    todo!()
}
