//! 基于数据库的ID生成器演示程序

use database::{DatabaseConfig, DatabaseIdGenerator, DatabaseManager};
use std::sync::Arc;
use tokio::sync::Mutex;
use utils::config::AppConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("基于数据库的ID生成器演示程序");

    // 从配置文件加载数据库配置
    let config = AppConfig::from_file("config/cu-scanner.toml")?;
    let db_config = DatabaseConfig::new(
        &config.database.host,
        config.database.port,
        &config.database.database,
        &config.database.username,
        &config.database.password,
    );

    // 创建数据库管理器
    let db_manager = Arc::new(Mutex::new(DatabaseManager::new(&db_config).await?));

    // 初始化数据库表
    {
        let mut db = db_manager.lock().await;
        db.init_tables().await?;
    }

    // 创建基于数据库的ID生成器
    let mut id_generator =
        DatabaseIdGenerator::new(db_manager.clone(), "demo_generator".to_string(), 10000);

    // 获取当前计数器值
    let current_counter = id_generator.get_current_counter().await?;
    println!("当前计数器值: {}", current_counter);

    // 为软件包生成对象ID
    println!("为软件包生成对象ID:");
    let obj_id1 = id_generator
        .generate_object_id_for_package("test-package-1")
        .await?;
    println!("  test-package-1: {}", obj_id1);

    let obj_id2 = id_generator
        .generate_object_id_for_package("test-package-2")
        .await?;
    println!("  test-package-2: {}", obj_id2);

    // 再次为相同软件包生成ID，应该返回相同的ID
    let obj_id3 = id_generator
        .generate_object_id_for_package("test-package-1")
        .await?;
    println!("  test-package-1 (重复): {}", obj_id3);
    assert_eq!(obj_id1, obj_id3);

    // 为EVR生成状态ID
    println!("为EVR生成状态ID:");
    let state_id1 = id_generator.generate_state_id_for_evr("1.0-1").await?;
    println!("  1.0-1: {}", state_id1);

    let state_id2 = id_generator.generate_state_id_for_evr("2.0-1").await?;
    println!("  2.0-1: {}", state_id2);

    // 再次为相同EVR生成ID，应该返回相同的ID
    let state_id3 = id_generator.generate_state_id_for_evr("1.0-1").await?;
    println!("  1.0-1 (重复): {}", state_id3);
    assert_eq!(state_id1, state_id3);

    // 为测试生成ID
    println!("为测试生成ID:");
    let test_id1 = id_generator
        .generate_test_id("test-package-1", "1.0-1")
        .await?;
    println!("  test-package-1:1.0-1: {}", test_id1);

    let test_id2 = id_generator
        .generate_test_id("test-package-2", "2.0-1")
        .await?;
    println!("  test-package-2:2.0-1: {}", test_id2);

    // 再次为相同测试生成ID，应该返回相同的ID
    let test_id3 = id_generator
        .generate_test_id("test-package-1", "1.0-1")
        .await?;
    println!("  test-package-1:1.0-1 (重复): {}", test_id3);
    assert_eq!(test_id1, test_id3);

    // 生成基本测试ID
    println!("生成基本测试ID:");
    let base_test_id1 = id_generator.generate_base_test_id("os_installed").await?;
    println!("  os_installed: {}", base_test_id1);

    let base_test_id2 = id_generator.generate_base_test_id("os_required").await?;
    println!("  os_required: {}", base_test_id2);

    // 最终计数器值
    let current_counter = id_generator.get_current_counter().await?;
    println!("最终计数器值: {}", current_counter);

    println!("基于数据库的ID生成器演示完成");
    Ok(())
}
