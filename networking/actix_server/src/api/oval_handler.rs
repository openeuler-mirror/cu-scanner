//! OVAL相关的API接口

use actix_web::{HttpResponse, Responder, get, web};
use database::{DatabaseConfig, DatabaseManager};
use log::{error, info};
use oval::CU_LINUX_SA_DEF_PREFIX;
use serde::{Deserialize, Serialize};

/// 从OVAL ID提取数字部分并格式化为文件名格式
///
/// # 参数
///
/// * `oval_id` - OVAL ID，格式如 "oval:com.culinux:def:20251001"
///
/// # 返回值
///
/// 格式化后的文件名，如 "security-oval-2025-1001.xml"
///
/// # 示例
///
/// ```ignore
/// let filename = format_oval_id_to_filename("oval:com.culinux:def:20251001");
/// assert_eq!(filename, "security-oval-2025-1001.xml");
/// ```
fn format_oval_id_to_filename(oval_id: &str) -> String {
    // 从OVAL ID中提取最后的数字部分
    // 例如: "oval:com.culinux:def:20251001" -> "20251001"
    let numeric_id = oval_id.split(':').last().unwrap_or(oval_id);

    // 将数字按4位分组
    // 例如: "20251001" -> "2025-1001"
    let formatted_id = if numeric_id.len() > 4 {
        let (first_part, second_part) = numeric_id.split_at(numeric_id.len() - 4);
        format!("{}-{}", first_part, second_part)
    } else {
        numeric_id.to_string()
    };

    format!("security-oval-{}.xml", formatted_id)
}

/// 从文件名解析出OVAL ID
///
/// # 参数
///
/// * `filename` - 文件名，格式如 "security-oval-2025-1001.xml"
///
/// # 返回值
///
/// 解析出的OVAL ID，如 "oval:com.culinux:def:20251001"，如果解析失败返回None
///
/// # 示例
///
/// ```ignore
/// let oval_id = parse_filename_to_oval_id("security-oval-2025-1001.xml");
/// assert_eq!(oval_id, Some("oval:com.culinux:def:20251001".to_string()));
/// ```
fn parse_filename_to_oval_id(filename: &str) -> Option<String> {
    // 移除 "security-oval-" 前缀和 ".xml" 后缀
    let filename = filename.strip_prefix("security-oval-")?
        .strip_suffix(".xml")?;

    // 移除连字符，恢复原始数字
    // 例如: "2025-1001" -> "20251001"
    let numeric_id = filename.replace('-', "");

    // 构造完整的OVAL ID
    Some(format!("{}{}", CU_LINUX_SA_DEF_PREFIX, numeric_id))
}

/// OVAL文件信息
#[derive(Serialize, Deserialize)]
pub struct OvalFileInfo {
    /// 文件ID
    pub id: String,
    /// 下载链接
    pub download_url: String,
    /// 文件大小（字节）
    pub size: Option<u64>,
}

/// OVAL文件列表响应
#[derive(Serialize, Deserialize)]
pub struct OvalFileListResponse {
    /// 文件总数
    pub total_count: usize,
    /// 文件列表
    pub files: Vec<OvalFileInfo>,
}

/// 获取所有OVAL文件的列表
///
/// # 返回值
///
/// 返回所有OVAL定义ID的列表，以及对应的下载链接等信息
#[get("/get_all")]
pub async fn get_all_oval_files(db_config: web::Data<DatabaseConfig>) -> impl Responder {
    info!("收到获取所有OVAL文件列表请求");

    // 连接数据库
    let db_manager = match DatabaseManager::new(&db_config).await {
        Ok(manager) => manager,
        Err(e) => {
            error!("数据库连接失败: {:?}", e);
            return HttpResponse::InternalServerError().json("数据库连接失败");
        }
    };

    info!("成功连接到数据库");

    // 查询所有OVAL定义
    let definitions = match db_manager.list_all_oval_definitions().await {
        Ok(definitions) => definitions,
        Err(e) => {
            error!("查询OVAL定义失败: {:?}", e);
            return HttpResponse::InternalServerError().json("查询OVAL定义失败");
        }
    };

    info!("查询到 {} 个OVAL定义", definitions.len());

    // 构建响应内容
    let mut files = Vec::new();
    for definition in definitions {
        let filename = format_oval_id_to_filename(&definition.id);
        let file_info = OvalFileInfo {
            id: definition.id.clone(),
            download_url: format!("/api/oval/each_file/{}", filename),
            size: None, // 文件大小需要额外查询，暂时设置为None
        };
        files.push(file_info);
    }

    let response = OvalFileListResponse {
        total_count: files.len(),
        files,
    };

    info!("返回JSON响应，包含 {} 个文件信息", response.total_count);
    HttpResponse::Ok().json(response)
}

/// 获取单个OVAL文件
///
/// # 参数
///
/// * `file_name` - 文件名
///
/// # 返回值
///
/// 返回OVAL文件内容
#[get("/each_file/{file_name}")]
pub async fn get_single_oval_file(
    file_name: web::Path<String>,
    db_config: web::Data<DatabaseConfig>,
) -> impl Responder {
    info!("收到获取单个OVAL文件请求，文件名: {}", file_name);

    // 从文件名解析出OVAL ID
    let oval_id = match parse_filename_to_oval_id(&file_name) {
        Some(id) => id,
        None => {
            error!("无效的文件名格式: {}", file_name);
            return HttpResponse::BadRequest().body("无效的文件名格式");
        }
    };

    info!("解析出OVAL ID: {}", oval_id);

    // 连接数据库
    let db_manager = match DatabaseManager::new(&db_config).await {
        Ok(manager) => manager,
        Err(e) => {
            error!("数据库连接失败: {:?}", e);
            return HttpResponse::InternalServerError().body("数据库连接失败");
        }
    };

    info!("成功连接到数据库");

    // 从数据库获取OVAL XML内容
    let xml_content = match db_manager.get_oval_xml_by_id(&oval_id).await {
        Ok(Some(content)) => content,
        Ok(None) => {
            info!("未找到指定的OVAL文件: {} (OVAL ID: {})", file_name, oval_id);
            return HttpResponse::NotFound().body("未找到指定的OVAL文件");
        }
        Err(e) => {
            error!("获取OVAL文件失败: {:?}", e);
            return HttpResponse::InternalServerError().body("获取OVAL文件失败");
        }
    };

    info!("成功获取OVAL文件内容，长度: {} 字节", xml_content.len());
    HttpResponse::Ok()
        .content_type("application/xml")
        .body(xml_content)
}

/// 配置OVAL处理相关的路由
pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(get_all_oval_files)
        .service(get_single_oval_file);
}
