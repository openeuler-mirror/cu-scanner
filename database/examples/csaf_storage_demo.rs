//! CSAF到OVAL转换并存储到数据库的演示程序

use csaf::CSAF;
use database::{DatabaseConfig, DatabaseManager};
use parser::csaf_to_oval;
use utils::config::AppConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("CSAF到OVAL转换并存储到数据库演示");

    // 从配置文件加载数据库配置
    println!("加载配置文件...");
    let config =
        AppConfig::from_file("/home/fatmouse/workspace/cu-scanner/config/cu-scanner.toml")?;
    let db_config = &config.database;

    // 创建数据库管理器配置
    let db_manager_config = DatabaseConfig::new(
        &db_config.host,
        db_config.port,
        &db_config.database,
        &db_config.username,
        &db_config.password,
    );

    // 创建数据库管理器
    let mut db_manager = DatabaseManager::new(&db_manager_config).await?;

    // 初始化数据库表
    db_manager.init_tables().await?;
    println!("数据库表初始化完成");

    // 加载CSAF测试文件
    println!("加载CSAF测试文件...");
    let csaf = CSAF::from_file(
        "/home/fatmouse/workspace/cu-scanner/test/csaf/csaf-openeuler-sa-2025-1004.json",
    )
    .map_err(|e| format!("加载CSAF文件失败: {}", e))?;
    println!("CSAF文件加载成功: {}", csaf.document.title);

    // 使用默认计数器转换CSAF到OVAL
    println!("转换CSAF到OVAL格式...");
    let oval = csaf_to_oval(&csaf).map_err(|e| format!("CSAF到OVAL转换失败: {}", e))?;
    println!("CSAF到OVAL转换成功");

    // 获取第一个定义（示例中只有一个定义）
    if let Some(definition) = oval.definitions.items.first() {
        println!("处理OVAL定义: {}", definition.metadata.title);

        // 转换完整OVAL定义
        let (db_definition, references, cves, rpminfo_tests, rpminfo_objects, rpminfo_states) =
            database::converter::convert_full_oval_definition(
                definition,
                &oval.tests,
                &oval.objects,
                &oval.states,
            );

        // 保存完整的OVAL定义到数据库
        println!("保存OVAL定义到数据库...");
        db_manager
            .save_full_oval_definition(
                &db_definition,
                &references,
                &cves,
                &rpminfo_tests,
                &rpminfo_objects,
                &rpminfo_states,
            )
            .await
            .map_err(|e| format!("保存OVAL定义到数据库失败: {}", e))?;
        println!("OVAL定义已成功保存到数据库");

        // 从数据库获取并验证保存的数据
        println!("从数据库获取OVAL定义...");
        let result = db_manager
            .get_full_oval_definition(&db_definition.id)
            .await
            .map_err(|e| format!("从数据库获取OVAL定义失败: {}", e))?;
        if let Some((
            retrieved_definition,
            retrieved_references,
            retrieved_cves,
            retrieved_tests,
            retrieved_objects,
            retrieved_states,
        )) = result
        {
            println!("成功从数据库获取OVAL定义: {}", retrieved_definition.title);
            println!("关联的引用数量: {}", retrieved_references.len());
            println!("关联的CVE数量: {}", retrieved_cves.len());
            println!("测试数量: {}", retrieved_tests.len());
            println!("对象数量: {}", retrieved_objects.len());
            println!("状态数量: {}", retrieved_states.len());
        } else {
            println!("未能从数据库获取OVAL定义");
        }
    } else {
        println!("OVAL定义列表为空");
    }

    Ok(())
}
