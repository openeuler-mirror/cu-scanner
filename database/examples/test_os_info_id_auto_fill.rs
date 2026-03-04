//! 测试os_info_id自动填充功能的示例程序

use database::{
    DatabaseConfig, DatabaseManager, OvalDefinition, RpmInfoObject, RpmInfoState, RpmInfoTest,
};
use utils::config::AppConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("测试os_info_id自动填充功能");

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
    let mut db_manager = DatabaseManager::new(&db_config).await?;

    println!("\n=== 测试1: 创建不带os_info_id的OVAL定义 ===");

    // 创建一个测试用的OVAL定义（os_info_id为None）
    let definition = OvalDefinition {
        id: "oval:test:def:1001".to_string(),
        class: "patch".to_string(),
        version: 1,
        title: "Test Definition for OS Info ID Auto Fill".to_string(),
        description: "Testing automatic os_info_id population".to_string(),
        family: "unix".to_string(),
        platform: "openEuler 20.03 LTS".to_string(),
        severity: "Important".to_string(),
        rights: "Copyright".to_string(),
        from: "test@example.com".to_string(),
        issued_date: "2025-01-01".to_string(),
        updated_date: "2025-01-01".to_string(),
        os_info_id: None, // 故意设置为None，测试自动填充
    };

    println!(
        "创建的定义: ID={}, os_info_id={:?}",
        definition.id, definition.os_info_id
    );

    // 创建包含oe1标识的RPM对象（应该匹配到openEuler 20.03）
    let rpminfo_objects = vec![RpmInfoObject {
        id: None,
        object_id: "oval:test:obj:1001".to_string(),
        ver: 1,
        rpm_name: "test-package".to_string(), // 包名不包含dist
    }];

    println!("RPM对象: rpm_name={}", rpminfo_objects[0].rpm_name);

    let references = vec![];
    let cves = vec![];
    let rpminfo_tests = vec![RpmInfoTest {
        check: "at least one".to_string(),
        comment: "test package check".to_string(),
        test_id: "oval:test:tst:1001".to_string(),
        version: 1,
        object_ref: "oval:test:obj:1001".to_string(),
        state_ref: "oval:test:ste:1001".to_string(),
    }];
    let rpminfo_states = vec![RpmInfoState {
        state_id: "oval:test:ste:1001".to_string(),
        version: "1".to_string(),
        evr_datatype: Some("evr_string".to_string()),
        evr_operation: Some("less than".to_string()),
        evr_value: Some("1.0-1.oe1".to_string()), // dist在EVR中
    }];

    println!("RPM状态: evr_value={:?}", rpminfo_states[0].evr_value);
    println!("\ndist应该从EVR中提取: 1.0-1.oe1 -> oe1");

    // 保存到数据库（应该自动填充os_info_id）
    println!("\n保存到数据库...");
    db_manager
        .save_full_oval_definition(
            &definition,
            &references,
            &cves,
            &rpminfo_tests,
            &rpminfo_objects,
            &rpminfo_states,
        )
        .await?;

    println!("✓ 保存成功!");

    // 从数据库读取并验证os_info_id是否已自动填充
    println!("\n=== 测试2: 验证os_info_id是否已填充 ===");
    if let Some(saved_def) = db_manager.get_oval_definition(&definition.id).await? {
        println!("从数据库读取的定义:");
        println!("  ID: {}", saved_def.id);
        println!("  Title: {}", saved_def.title);
        println!("  os_info_id: {:?}", saved_def.os_info_id);

        if let Some(os_info_id) = saved_def.os_info_id {
            println!("\n✓ os_info_id已自动填充: {}", os_info_id);

            // 查询对应的OS信息
            if let Some(os_info) = db_manager.get_os_info_by_id(os_info_id).await? {
                println!("\n对应的OS信息:");
                println!("  OS类型: {}", os_info.os_type);
                println!("  OS版本: {}", os_info.os_version);
                println!("  Dist: {}", os_info.dist);
                println!("  描述: {:?}", os_info.description);
            }
        } else {
            println!("\n✗ os_info_id未填充（仍为None）");
        }
    } else {
        println!("✗ 无法从数据库读取定义");
    }

    println!("\n=== 测试3: 测试oe2203标识 ===");
    let definition2 = OvalDefinition {
        id: "oval:test:def:1002".to_string(),
        class: "patch".to_string(),
        version: 1,
        title: "Test Definition for oe2203".to_string(),
        description: "Testing oe2203 dist".to_string(),
        family: "unix".to_string(),
        platform: "openEuler 22.03 LTS".to_string(),
        severity: "Important".to_string(),
        rights: "Copyright".to_string(),
        from: "test@example.com".to_string(),
        issued_date: "2025-01-01".to_string(),
        updated_date: "2025-01-01".to_string(),
        os_info_id: None,
    };

    let rpminfo_objects2 = vec![RpmInfoObject {
        id: None,
        object_id: "oval:test:obj:1002".to_string(),
        ver: 1,
        rpm_name: "kernel".to_string(), // 包名不包含dist
    }];

    let rpminfo_states2 = vec![RpmInfoState {
        state_id: "oval:test:ste:1002".to_string(),
        version: "1".to_string(),
        evr_datatype: Some("evr_string".to_string()),
        evr_operation: Some("less than".to_string()),
        evr_value: Some("5.10.0-136.12.0.86.oe2203".to_string()), // dist在这里
    }];

    db_manager
        .save_full_oval_definition(
            &definition2,
            &vec![],
            &vec![],
            &vec![],
            &rpminfo_objects2,
            &rpminfo_states2,
        )
        .await?;

    if let Some(saved_def2) = db_manager.get_oval_definition(&definition2.id).await? {
        println!("定义2的os_info_id: {:?}", saved_def2.os_info_id);
        if let Some(os_info_id) = saved_def2.os_info_id {
            if let Some(os_info) = db_manager.get_os_info_by_id(os_info_id).await? {
                println!(
                    "  匹配到: {} {} (dist: {})",
                    os_info.os_type, os_info.os_version, os_info.dist
                );
            }
        }
    }

    println!("\n✓ 所有测试完成!");
    Ok(())
}
