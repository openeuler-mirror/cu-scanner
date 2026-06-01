//! 测试OS信息查询功能的示例程序

use database::{DatabaseConfig, DatabaseManager};
use utils::config::AppConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("测试OS信息查询功能");

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

    // 测试1: 列出所有OS信息
    println!("\n=== 测试1: 列出所有OS信息 ===");
    let os_infos = db_manager.list_all_os_info().await?;
    for os_info in &os_infos {
        println!(
            "ID: {:?}, OS: {} {}, Dist: {}, Package: {}, Verify File: {}",
            os_info.id,
            os_info.os_type,
            os_info.os_version,
            os_info.dist,
            os_info.package_name,
            os_info.verify_file
        );
    }

    // 测试2: 根据dist查找OS信息
    println!("\n=== 测试2: 根据dist查找OS信息 ===");
    let test_dists = vec!["oe1", "oe2203", "el7", "el8"];
    for dist in test_dists {
        if let Some(os_info) = db_manager.find_os_info_by_dist(dist).await? {
            println!(
                "Found {} -> {} {}",
                dist, os_info.os_type, os_info.os_version
            );
        } else {
            println!("Not found: {}", dist);
        }
    }

    // 测试3: 从软件包版本中提取并匹配OS信息
    println!("\n=== 测试3: 从软件包版本提取dist并匹配OS ===");
    let test_packages = vec![
        "ansible-2.9-1.oe1",
        "kernel-5.10.0-136.12.0.86.oe2203",
        "systemd-239-58.el7",
        "glibc-2.28-151.el8",
        "unknown-package-1.0.fc35", // 不存在的dist
    ];

    for package in test_packages {
        println!("\n软件包: {}", package);
        match db_manager.extract_and_match_os_info(package).await? {
            Some(os_info) => {
                println!("  ✓ 匹配成功!");
                println!("    OS类型: {}", os_info.os_type);
                println!("    OS版本: {}", os_info.os_version);
                println!("    Dist: {}", os_info.dist);
                println!("    验证文件: {}", os_info.verify_file);
            }
            None => {
                println!("  ✗ 未找到匹配的OS信息");
            }
        }
    }

    println!("\n所有测试完成!");
    Ok(())
}
