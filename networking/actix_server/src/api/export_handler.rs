//! OVAL批量导出相关的API接口

use actix_web::{HttpResponse, Responder, get, web};
use database::{DatabaseConfig, DatabaseManager};
use log::{error, info, warn};
use serde::Deserialize;

/// 导出查询参数（用于系统类型过滤）
#[derive(Deserialize)]
pub struct ExportQuery {
    /// 操作系统类型（可选）
    pub os_type: Option<String>,
}

/// 日期范围查询参数
#[derive(Deserialize)]
pub struct RangeQuery {
    /// 开始日期
    pub start_date: String,
    /// 结束日期
    pub end_date: String,
    /// 操作系统类型（可选）
    pub os_type: Option<String>,
}

/// 按月导出OVAL定义
///
/// # 参数
///
/// * `year` - 年份
/// * `month` - 月份（1-12）
/// * `os_type` - 操作系统类型（可选，通过query参数传递）
///
/// # 返回值
///
/// 返回合并的OVAL XML文件
#[get("/monthly/{year}/{month}")]
pub async fn export_monthly(
    path: web::Path<(i32, u32)>,
    query: web::Query<ExportQuery>,
    db_config: web::Data<DatabaseConfig>,
) -> impl Responder {
    let (year, month) = path.into_inner();
    let os_type = query.os_type.as_deref();

    info!(
        "收到按月导出请求: {}-{:02}, 系统类型过滤: {:?}",
        year, month, os_type
    );

    // 参数验证
    if !(1..=12).contains(&month) {
        warn!("无效的月份参数: {}", month);
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "无效的月份参数",
            "details": "月份必须在1-12之间"
        }));
    }

    // 连接数据库
    let db_manager = match DatabaseManager::new(&db_config).await {
        Ok(manager) => manager,
        Err(e) => {
            error!("数据库连接失败: {:?}", e);
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "数据库连接失败"
            }));
        }
    };

    // 查询并导出
    let merged = match db_manager.export_oval_by_month(year, month, os_type).await {
        Ok(oval) => oval,
        Err(e) => {
            error!("导出OVAL定义失败: {:?}", e);
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "导出OVAL定义失败",
                "details": format!("{:?}", e)
            }));
        }
    };

    // 检查是否有数据
    if merged.is_empty() {
        info!("未找到匹配的OVAL定义");
        return HttpResponse::Ok().json(serde_json::json!({
            "message": "未找到匹配的OVAL定义",
            "filters": {
                "year": year,
                "month": month,
                "os_type": os_type
            }
        }));
    }

    // 转换为XML
    let xml_content = match merged.to_oval_string() {
        Ok(xml) => xml,
        Err(e) => {
            error!("转换为XML失败: {:?}", e);
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "转换为XML失败"
            }));
        }
    };

    // 生成文件名
    let filename = if let Some(os) = os_type {
        format!("oval-{}-{:02}-{}.xml", year, month, os)
    } else {
        format!("oval-{}-{:02}.xml", year, month)
    };

    info!(
        "成功导出OVAL定义，包含 {} 个definitions，文件名: {}",
        merged.get_definition_count(),
        filename
    );

    HttpResponse::Ok()
        .content_type("application/xml")
        .insert_header(("Content-Disposition", format!("attachment; filename=\"{}\"", filename)))
        .body(xml_content)
}

/// 按周导出OVAL定义
///
/// # 参数
///
/// * `year` - 年份
/// * `week` - ISO周号（1-53）
/// * `os_type` - 操作系统类型（可选，通过query参数传递）
///
/// # 返回值
///
/// 返回合并的OVAL XML文件
#[get("/weekly/{year}/{week}")]
pub async fn export_weekly(
    path: web::Path<(i32, u32)>,
    query: web::Query<ExportQuery>,
    db_config: web::Data<DatabaseConfig>,
) -> impl Responder {
    let (year, week) = path.into_inner();
    let os_type = query.os_type.as_deref();

    info!(
        "收到按周导出请求: {}-W{:02}, 系统类型过滤: {:?}",
        year, week, os_type
    );

    // 参数验证
    if !(1..=53).contains(&week) {
        warn!("无效的周数参数: {}", week);
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "无效的周数参数",
            "details": "周数必须在1-53之间"
        }));
    }

    // 连接数据库
    let db_manager = match DatabaseManager::new(&db_config).await {
        Ok(manager) => manager,
        Err(e) => {
            error!("数据库连接失败: {:?}", e);
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "数据库连接失败"
            }));
        }
    };

    // 查询并导出
    let merged = match db_manager.export_oval_by_week(year, week, os_type).await {
        Ok(oval) => oval,
        Err(e) => {
            error!("导出OVAL定义失败: {:?}", e);
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "导出OVAL定义失败",
                "details": format!("{:?}", e)
            }));
        }
    };

    // 检查是否有数据
    if merged.is_empty() {
        info!("未找到匹配的OVAL定义");
        return HttpResponse::Ok().json(serde_json::json!({
            "message": "未找到匹配的OVAL定义",
            "filters": {
                "year": year,
                "week": week,
                "os_type": os_type
            }
        }));
    }

    // 转换为XML
    let xml_content = match merged.to_oval_string() {
        Ok(xml) => xml,
        Err(e) => {
            error!("转换为XML失败: {:?}", e);
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "转换为XML失败"
            }));
        }
    };

    // 生成文件名
    let filename = if let Some(os) = os_type {
        format!("oval-{}-W{:02}-{}.xml", year, week, os)
    } else {
        format!("oval-{}-W{:02}.xml", year, week)
    };

    info!(
        "成功导出OVAL定义，包含 {} 个definitions，文件名: {}",
        merged.get_definition_count(),
        filename
    );

    HttpResponse::Ok()
        .content_type("application/xml")
        .insert_header(("Content-Disposition", format!("attachment; filename=\"{}\"", filename)))
        .body(xml_content)
}

/// 按年导出OVAL定义
///
/// # 参数
///
/// * `year` - 年份
/// * `os_type` - 操作系统类型（可选，通过query参数传递）
///
/// # 返回值
///
/// 返回合并的OVAL XML文件
#[get("/yearly/{year}")]
pub async fn export_yearly(
    path: web::Path<i32>,
    query: web::Query<ExportQuery>,
    db_config: web::Data<DatabaseConfig>,
) -> impl Responder {
    let year = path.into_inner();
    let os_type = query.os_type.as_deref();

    info!(
        "收到按年导出请求: {}, 系统类型过滤: {:?}",
        year, os_type
    );

    // 连接数据库
    let db_manager = match DatabaseManager::new(&db_config).await {
        Ok(manager) => manager,
        Err(e) => {
            error!("数据库连接失败: {:?}", e);
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "数据库连接失败"
            }));
        }
    };

    // 查询并导出
    let merged = match db_manager.export_oval_by_year(year, os_type).await {
        Ok(oval) => oval,
        Err(e) => {
            error!("导出OVAL定义失败: {:?}", e);
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "导出OVAL定义失败",
                "details": format!("{:?}", e)
            }));
        }
    };

    // 检查是否有数据
    if merged.is_empty() {
        info!("未找到匹配的OVAL定义");
        return HttpResponse::Ok().json(serde_json::json!({
            "message": "未找到匹配的OVAL定义",
            "filters": {
                "year": year,
                "os_type": os_type
            }
        }));
    }

    // 转换为XML
    let xml_content = match merged.to_oval_string() {
        Ok(xml) => xml,
        Err(e) => {
            error!("转换为XML失败: {:?}", e);
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "转换为XML失败"
            }));
        }
    };

    // 生成文件名
    let filename = if let Some(os) = os_type {
        format!("oval-{}-{}.xml", year, os)
    } else {
        format!("oval-{}.xml", year)
    };

    info!(
        "成功导出OVAL定义，包含 {} 个definitions，文件名: {}",
        merged.get_definition_count(),
        filename
    );

    HttpResponse::Ok()
        .content_type("application/xml")
        .insert_header(("Content-Disposition", format!("attachment; filename=\"{}\"", filename)))
        .body(xml_content)
}

/// 按日期范围导出OVAL定义
///
/// # Query参数
///
/// * `start_date` - 开始日期（YYYY-MM-DD）
/// * `end_date` - 结束日期（YYYY-MM-DD）
/// * `os_type` - 操作系统类型（可选）
///
/// # 返回值
///
/// 返回合并的OVAL XML文件
#[get("/range")]
pub async fn export_range(
    query: web::Query<RangeQuery>,
    db_config: web::Data<DatabaseConfig>,
) -> impl Responder {
    let start_date = &query.start_date;
    let end_date = &query.end_date;
    let os_type = query.os_type.as_deref();

    info!(
        "收到按日期范围导出请求: {} 到 {}, 系统类型过滤: {:?}",
        start_date, end_date, os_type
    );

    // 基本日期格式验证
    if start_date.len() != 10 || end_date.len() != 10 {
        warn!("无效的日期格式: {} - {}", start_date, end_date);
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "无效的日期格式",
            "details": "日期格式必须为YYYY-MM-DD"
        }));
    }

    // 验证开始日期不能晚于结束日期
    if start_date > end_date {
        warn!("开始日期晚于结束日期: {} > {}", start_date, end_date);
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "无效的日期范围",
            "details": "开始日期不能晚于结束日期"
        }));
    }

    // 连接数据库
    let db_manager = match DatabaseManager::new(&db_config).await {
        Ok(manager) => manager,
        Err(e) => {
            error!("数据库连接失败: {:?}", e);
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "数据库连接失败"
            }));
        }
    };

    // 查询并导出
    let merged = match db_manager.export_oval_by_time_and_os(start_date, end_date, os_type).await {
        Ok(oval) => oval,
        Err(e) => {
            error!("导出OVAL定义失败: {:?}", e);
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "导出OVAL定义失败",
                "details": format!("{:?}", e)
            }));
        }
    };

    // 检查是否有数据
    if merged.is_empty() {
        info!("未找到匹配的OVAL定义");
        return HttpResponse::Ok().json(serde_json::json!({
            "message": "未找到匹配的OVAL定义",
            "filters": {
                "start_date": start_date,
                "end_date": end_date,
                "os_type": os_type
            }
        }));
    }

    // 转换为XML
    let xml_content = match merged.to_oval_string() {
        Ok(xml) => xml,
        Err(e) => {
            error!("转换为XML失败: {:?}", e);
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "转换为XML失败"
            }));
        }
    };

    // 生成文件名
    let filename = if let Some(os) = os_type {
        format!("oval-{}-to-{}-{}.xml", start_date, end_date, os)
    } else {
        format!("oval-{}-to-{}.xml", start_date, end_date)
    };

    info!(
        "成功导出OVAL定义，包含 {} 个definitions，文件名: {}",
        merged.get_definition_count(),
        filename
    );

    HttpResponse::Ok()
        .content_type("application/xml")
        .insert_header(("Content-Disposition", format!("attachment; filename=\"{}\"", filename)))
        .body(xml_content)
}

/// 按操作系统类型导出所有OVAL定义
///
/// # 参数
///
/// * `os_type` - 操作系统类型
///
/// # 返回值
///
/// 返回合并的OVAL XML文件
#[get("/os/{os_type}")]
pub async fn export_by_os(
    path: web::Path<String>,
    db_config: web::Data<DatabaseConfig>,
) -> impl Responder {
    let os_type = path.into_inner();

    info!("收到按操作系统类型导出请求: {}", os_type);

    // 连接数据库
    let db_manager = match DatabaseManager::new(&db_config).await {
        Ok(manager) => manager,
        Err(e) => {
            error!("数据库连接失败: {:?}", e);
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "数据库连接失败"
            }));
        }
    };

    // 查询并导出
    let merged = match db_manager.export_oval_by_os_type(&os_type).await {
        Ok(oval) => oval,
        Err(e) => {
            error!("导出OVAL定义失败: {:?}", e);
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "导出OVAL定义失败",
                "details": format!("{:?}", e)
            }));
        }
    };

    // 检查是否有数据
    if merged.is_empty() {
        info!("未找到匹配的OVAL定义");
        return HttpResponse::Ok().json(serde_json::json!({
            "message": "未找到匹配的OVAL定义",
            "filters": {
                "os_type": os_type
            }
        }));
    }

    // 转换为XML
    let xml_content = match merged.to_oval_string() {
        Ok(xml) => xml,
        Err(e) => {
            error!("转换为XML失败: {:?}", e);
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "转换为XML失败"
            }));
        }
    };

    // 生成文件名
    let filename = format!("oval-{}.xml", os_type);

    info!(
        "成功导出OVAL定义，包含 {} 个definitions，文件名: {}",
        merged.get_definition_count(),
        filename
    );

    HttpResponse::Ok()
        .content_type("application/xml")
        .insert_header(("Content-Disposition", format!("attachment; filename=\"{}\"", filename)))
        .body(xml_content)
}

/// 配置导出相关的路由
pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(export_monthly)
        .service(export_weekly)
        .service(export_yearly)
        .service(export_range)
        .service(export_by_os);
}
