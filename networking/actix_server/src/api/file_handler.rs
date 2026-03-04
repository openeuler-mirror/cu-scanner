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

    // 转换CSAF到OVAL
    let oval = match csaf_to_oval(&csaf) {
        Ok(oval) => {
            info!("CSAF到OVAL转换成功");
            oval
        }
        Err(e) => {
            error!("CSAF到OVAL转换失败: {}", e);
            let response = UploadResponse {
                success: false,
                message: format!("CSAF到OVAL转换失败: {}", e),
                oval_id: None,
            };
            return HttpResponse::InternalServerError().json(response);
        }
    };

    // 获取OVAL定义ID
    let oval_id = if !oval.definitions.items.is_empty() {
        oval.definitions.items[0].id.clone()
    } else {
        error!("转换后的OVAL定义为空");
        let response = UploadResponse {
            success: false,
            message: "转换后的OVAL定义为空".to_string(),
            oval_id: None,
        };
        return HttpResponse::InternalServerError().json(response);
    };

    // 连接数据库
    let db_manager = match DatabaseManager::new(&db_config).await {
        Ok(manager) => manager,
        Err(e) => {
            error!("数据库连接失败: {:?}", e);
            let response = UploadResponse {
                success: false,
                message: format!("数据库连接失败: {:?}", e),
                oval_id: None,
            };
            return HttpResponse::InternalServerError().json(response);
        }
    };

    // 转换OVAL定义为数据库实体
    let (db_definition, references, cves, rpminfo_tests, rpminfo_objects, rpminfo_states) =
        database::converter::convert_full_oval_definition(
            &oval.definitions.items[0],
            &oval.tests,
            &oval.objects,
            &oval.states,
        );

    // 保存到数据库
    let mut db_manager = db_manager; // 重新绑定为可变引用
    match db_manager
        .save_full_oval_definition(
            &db_definition,
            &references,
            &cves,
            &rpminfo_tests,
            &rpminfo_objects,
            &rpminfo_states,
        )
        .await
    {
        Ok(_) => {
            info!("OVAL定义保存到数据库成功");
            let response = UploadResponse {
                success: true,
                message: "CSAF文件处理成功".to_string(),
                oval_id: Some(oval_id),
            };
            HttpResponse::Ok().json(response)
        }
        Err(e) => {
            error!("保存OVAL定义到数据库失败: {:?}", e);
            let response = UploadResponse {
                success: false,
                message: format!("保存OVAL定义到数据库失败: {:?}", e),
                oval_id: Some(oval_id), // 即使保存失败也返回ID
            };
            HttpResponse::InternalServerError().json(response)
        }
    }
}

/// 配置文件处理相关的路由
pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(post_csaf_file);
}
