//! 解析CSAF文件，转换为OVAL格式并存储到数据库的示例程序

use csaf::CSAF;
use database::{DatabaseConfig, DatabaseManager};
use parser::{csaf_to_oval, process_csaf_id};
use std::env;
use utils::config::AppConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("CSAF到数据库存储示例程序");

    // 获取命令行参数
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("使用方法: {} <csaf_file_path>", args[0]);
        std::process::exit(1);
    }

    let csaf_file_path = &args[1];
    println!("正在处理CSAF文件: {}", csaf_file_path);

    // 从配置文件加载数据库配置
    let config = AppConfig::from_file("config/cu-scanner.toml")
        .map_err(|e| format!("配置文件加载失败: {}", e))?;
    let db_config = DatabaseConfig::new(
        &config.database.host,
        config.database.port,
        &config.database.database,
        &config.database.username,
        &config.database.password,
    );

    // 连接数据库
    let mut db_manager = DatabaseManager::new(&db_config)
        .await
        .map_err(|e| format!("数据库连接失败: {:?}", e))?;

    // 初始化数据库表结构
    println!("正在初始化数据库表结构...");
    db_manager
        .init_tables()
        .await
        .map_err(|e| format!("数据库表初始化失败: {:?}", e))?;

    // 加载CSAF文件
    println!("正在加载CSAF文件...");
    let csaf = CSAF::from_file(csaf_file_path).map_err(|e| format!("CSAF文件加载失败: {}", e))?;
    println!("成功加载CSAF文件: {}", csaf.document.title);

    // 转换为OVAL格式
    println!("正在将CSAF转换为OVAL格式...");
    let oval = csaf_to_oval(&csaf).map_err(|e| format!("CSAF到OVAL转换失败: {}", e))?;
    println!("CSAF到OVAL转换完成");

    // 获取第一个定义（通常一个CSAF文件只有一个定义）
    if !oval.definitions.items.is_empty() {
        let definition = &oval.definitions.items[0];
        println!("处理的OVAL定义ID: {}", definition.id);

        // 处理CSAF ID以生成数据库ID
        let processed_id = process_csaf_id(&csaf.document.tracking.id);
        let db_definition_id = format!("oval:cn.chinaunicom.culinux.cusa:def:{}", processed_id);

        // 创建数据库实体
        let db_definition = database::OvalDefinition {
            id: db_definition_id.clone(),
            class: "patch".to_string(),
            version: csaf.document.tracking.version.parse().unwrap_or(1),
            title: definition.metadata.title.clone(),
            description: definition.metadata.description.clone(),
            family: definition.metadata.affected.family.clone(),
            platform: definition.metadata.affected.platform.clone(),
            severity: definition.metadata.advisory.severity.clone(),
            rights: definition.metadata.advisory.rights.clone(),
            from: definition.metadata.advisory.from.clone(),
            issued_date: definition.metadata.advisory.issued.date.clone(),
            updated_date: definition.metadata.advisory.updated.date.clone(),
            os_info_id: None, // 将在存储时根据软件包版本匹配
        };

        // 创建引用列表
        let mut db_references = Vec::new();
        if let Some(refs) = &definition.metadata.references {
            for reference in refs {
                db_references.push(database::Reference {
                    ref_id: reference.ref_id.clone(),
                    ref_url: reference.ref_url.clone(),
                    source: reference.source.clone(),
                });
            }
        }

        // 创建CVE列表
        let mut db_cves = Vec::new();
        for cve in &definition.metadata.advisory.cve {
            db_cves.push(database::Cve {
                cve_id: cve.content.clone(),
                cvss3: cve.cvss3.clone(),
                impact: cve.impact.clone(),
                href: cve.href.clone(),
                content: cve.content.clone(),
            });
        }

        // 从OVAL中提取测试、对象和状态信息
        let mut db_rpminfo_tests = Vec::new();
        let mut db_rpminfo_objects = Vec::new();
        let mut db_rpminfo_states = Vec::new();

        // 提取测试信息
        for test in &oval.tests.rpminfo_tests {
            db_rpminfo_tests.push(database::RpmInfoTest {
                check: test.check.clone(),
                comment: test.comment.clone(),
                test_id: test.id.clone(),
                version: test.version,
                object_ref: test.object.object_ref.clone(),
                state_ref: test.state.state_ref.clone(),
            });
        }

        // 提取对象信息
        for object in &oval.objects.rpm_info_objects {
            db_rpminfo_objects.push(database::RpmInfoObject {
                id: None, // 数据库自增ID，在保存时由数据库生成
                object_id: object.id.to_string(),
                ver: object.ver,
                rpm_name: object.rpm_name.clone(),
            });
        }

        // 提取状态信息
        if let Some(states) = &oval.states.rpminfo_states {
            for state in states {
                db_rpminfo_states.push(database::RpmInfoState {
                    state_id: state.id.clone(),
                    version: state.version.clone(),
                    evr_datatype: state.evr.as_ref().map(|e| e.datatype.clone()),
                    evr_operation: state.evr.as_ref().map(|e| e.operation.clone()),
                    evr_value: state.evr.as_ref().map(|e| e.evr.clone()),
                });
            }
        }

        // 保存到数据库
        println!("正在保存OVAL定义到数据库...");
        println!("  - 引用数量: {}", db_references.len());
        println!("  - CVE数量: {}", db_cves.len());
        println!("  - 测试数量: {}", db_rpminfo_tests.len());
        println!("  - 对象数量: {}", db_rpminfo_objects.len());
        println!("  - 状态数量: {}", db_rpminfo_states.len());

        db_manager
            .save_full_oval_definition(
                &db_definition,
                &db_references,
                &db_cves,
                &db_rpminfo_tests,
                &db_rpminfo_objects,
                &db_rpminfo_states,
            )
            .await
            .map_err(|e| format!("保存到数据库失败: {:?}", e))?;

        println!("成功将OVAL定义保存到数据库，ID: {}", db_definition_id);
    } else {
        println!("OVAL定义列表为空，没有数据需要保存");
    }

    Ok(())
}
