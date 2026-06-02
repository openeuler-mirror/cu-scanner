//! 测试从数据库导出OVAL XML时是否包含OS检查信息

use database::{DatabaseConfig, DatabaseManager};
use utils::config::AppConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("测试从数据库导出OVAL XML时是否包含OS检查信息\n");

    // 从配置文件加载数据库配置
    let config = AppConfig::from_file("config/cu-scanner.toml")?;
    let db_config = DatabaseConfig::new(
        &config.database.host,
        config.database.port,
        &config.database.database,
        &config.database.username,
        &config.database.password,
    );

    // 连接数据库
    let db_manager = DatabaseManager::new(&db_config).await?;

    // 列出所有OVAL定义
    println!("正在查询所有OVAL定义...");
    let definitions = db_manager.list_all_oval_definitions().await?;

    if definitions.is_empty() {
        println!("数据库中没有OVAL定义");
        return Ok(());
    }

    println!("找到 {} 个OVAL定义\n", definitions.len());

    // 取第一个有os_info_id的定义进行测试
    let test_definition = definitions
        .iter()
        .find(|d| d.os_info_id.is_some())
        .or_else(|| definitions.first());

    if let Some(def) = test_definition {
        println!("测试定义: {}", def.id);
        println!("  标题: {}", def.title);
        println!("  平台: {}", def.platform);
        println!("  os_info_id: {:?}", def.os_info_id);

        // 如果有os_info_id，查询OS信息
        if let Some(os_info_id) = def.os_info_id {
            if let Ok(Some(os_info)) = db_manager.get_os_info_by_id(os_info_id).await {
                println!("  OS类型: {} {}", os_info.os_type, os_info.os_version);
                println!("  dist: {}", os_info.dist);
            }
        }

        println!("\n正在导出为OVAL XML...");
        match db_manager.get_oval_xml_by_id(&def.id).await? {
            Some(xml_content) => {
                println!("✓ 成功导出XML，长度: {} 字节\n", xml_content.len());

                // 检查XML内容
                println!("检查XML内容:");

                // 检查是否包含rpmverifyfile_test
                let has_rpmverifyfile_test = xml_content.contains("rpmverifyfile_test");
                println!("  ✓ 包含 rpmverifyfile_test: {}", has_rpmverifyfile_test);

                // 检查是否包含rpmverifyfile_object
                let has_rpmverifyfile_object = xml_content.contains("rpmverifyfile_object");
                println!(
                    "  ✓ 包含 rpmverifyfile_object: {}",
                    has_rpmverifyfile_object
                );

                // 检查是否包含rpmverifyfile_state
                let has_rpmverifyfile_state = xml_content.contains("rpmverifyfile_state");
                println!("  ✓ 包含 rpmverifyfile_state: {}", has_rpmverifyfile_state);

                // 检查是否包含 "must be installed"
                let has_must_be_installed = xml_content.contains("must be installed");
                println!("  ✓ 包含 'must be installed': {}", has_must_be_installed);

                // 检查是否包含 "is installed"
                let has_is_installed = xml_content.contains("is installed");
                println!("  ✓ 包含 'is installed': {}", has_is_installed);

                // 检查是否包含固定的OS检查ID
                let has_os_test_1 = xml_content.contains("oval:cn.chinaunicom.culinux.cusa:tst:1");
                let has_os_test_2 = xml_content.contains("oval:cn.chinaunicom.culinux.cusa:tst:2");
                let has_os_obj = xml_content.contains("oval:cn.chinaunicom.culinux.cusa:obj:1");
                let has_os_state = xml_content.contains("oval:cn.chinaunicom.culinux.cusa:ste:1");

                println!("  ✓ 包含 OS test 1 (tst:1): {}", has_os_test_1);
                println!("  ✓ 包含 OS test 2 (tst:2): {}", has_os_test_2);
                println!("  ✓ 包含 OS object (obj:1): {}", has_os_obj);
                println!("  ✓ 包含 OS state (ste:1): {}", has_os_state);

                // 保存到文件
                let filename = format!("test_export_{}.xml", def.id.replace(":", "_"));
                std::fs::write(&filename, &xml_content)?;
                println!("\n已保存到文件: {}", filename);

                // 显示部分XML内容
                println!("\nXML内容预览（前1000个字符）:");
                println!("{}", &xml_content[..std::cmp::min(1000, xml_content.len())]);

                if xml_content.len() > 1000 {
                    println!("...");
                    println!("\nXML内容预览（最后500个字符）:");
                    let start = xml_content.len().saturating_sub(500);
                    println!("{}", &xml_content[start..]);
                }
            }
            None => {
                println!("未找到指定ID的OVAL定义");
            }
        }
    } else {
        println!("没有找到可以测试的定义");
    }

    Ok(())
}
