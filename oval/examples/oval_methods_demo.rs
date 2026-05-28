//! OVAL 数据结构方法使用示例
//!
//! 该示例程序演示了如何使用 OVAL 数据结构的各种便捷方法。

use oval::*;

fn main() -> Result<()> {
    println!("=== OVAL 数据结构方法使用示例 ===\n");

    // 创建 OVAL 定义
    let mut oval = OvalDefinitions::new();
    // 使用RFC3339格式（符合xs:dateTime要求）
    oval.generator.time_stamp = "2024-01-01T12:00:00Z".to_string();

    println!("【1. 初始状态检查】");
    println!("  定义数量: {}", oval.get_definition_count());
    println!("  测试数量: {}", oval.get_test_count());
    println!("  对象数量: {}", oval.get_object_count());
    println!("  状态数量: {}", oval.get_state_count());
    println!("  是否为空: {}", oval.is_empty());
    println!();

    // 创建元数据
    println!("【2. 创建元数据】");
    let mut metadata = Metadata::new();
    metadata.title = "nginx 安全更新".to_string();
    metadata.description = "修复 nginx 中的安全漏洞".to_string();
    metadata.affected.platform = "China Unicom Linux 4".to_string();

    // 添加引用
    let reference = Reference::new();
    metadata.add_reference(reference);
    println!("  标题: {}", metadata.title);
    println!("  引用数量: {}", metadata.get_reference_count());

    // 添加 CVE
    let cve1 = CVE::new()
        .with_content("CVE-2024-1234".to_string())
        .with_cvss3("7.5".to_string())
        .with_href("https://nvd.nist.gov/vuln/detail/CVE-2024-1234".to_string())
        .with_impact("High".to_string());

    let cve2 = CVE::new()
        .with_content("CVE-2024-5678".to_string())
        .with_cvss3("9.8".to_string())
        .with_href("https://nvd.nist.gov/vuln/detail/CVE-2024-5678".to_string())
        .with_impact("Critical".to_string());

    metadata.advisory.add_cve(cve1);
    metadata.advisory.add_cve(cve2);
    metadata.advisory.severity = "High".to_string();
    metadata.advisory.issued.date = "2024-01-01".to_string();
    metadata.advisory.updated.date = "2024-01-02".to_string();

    println!("  CVE 数量: {}", metadata.advisory.get_cve_count());
    println!("  CVE 列表: {:?}", metadata.advisory.get_cve_ids());
    println!(
        "  包含 CVE-2024-1234: {}",
        metadata.advisory.contains_cve("CVE-2024-1234")
    );
    println!(
        "  包含 CVE-9999-9999: {}",
        metadata.advisory.contains_cve("CVE-9999-9999")
    );
    println!();

    // 创建条件
    println!("【3. 创建检查条件】");
    let mut criteria = Criteria::new().with_operator("AND".to_string());

    let criterion1 = Criterion::new();
    let criterion2 = Criterion::new();
    criteria.add_criterion(criterion1);
    criteria.add_criterion(criterion2);

    println!("  操作符: {}", criteria.operator);
    println!("  条件数量: {}", criteria.get_criterion_count());
    println!("  子条件数量: {}", criteria.get_sub_criteria_count());
    println!();

    // 创建定义
    println!("【4. 创建完整定义】");
    let definition = Definition::new()
        .with_id("oval:cn.chinaunicom.culinux.cusa:def:20241001".to_string())
        .with_class("patch".to_string())
        .with_version(1)
        .with_metadata(metadata)
        .with_criteria(criteria);

    println!("  定义 ID: {}", definition.get_id());
    println!("  定义标题: {}", definition.get_title());
    println!("  定义类别: {}", definition.class);
    println!("  定义版本: {}", definition.version);

    oval.add_definition(definition);
    println!("  添加后定义数量: {}", oval.get_definition_count());
    println!();

    // 创建测试
    println!("【5. 创建 RPM 信息测试】");
    let test = RpmInfoTest::new()
        .with_id("oval:cn.chinaunicom.culinux.cusa:tst:20241001".to_string())
        .with_check("all".to_string())
        .with_comment("检查 nginx 软件包版本是否小于修复版本".to_string())
        .with_version(1)
        .with_object_ref("oval:cn.chinaunicom.culinux.cusa:obj:20241001".to_string())
        .with_state_ref("oval:cn.chinaunicom.culinux.cusa:ste:20241001".to_string());

    println!("  测试 ID: {}", test.id);
    println!("  检查方式: {}", test.check);
    println!("  注释: {}", test.comment);
    println!("  对象引用: {}", test.object.object_ref);
    println!("  状态引用: {}", test.state.state_ref);

    oval.add_rpminfo_test(test);
    println!("  添加后测试数量: {}", oval.get_test_count());
    println!();

    // 创建对象
    println!("【6. 创建 RPM 信息对象】");
    let object = RpmInfoObject::new()
        .with_id("oval:cn.chinaunicom.culinux.cusa:obj:20241001".to_string())
        .with_ver(1)
        .with_rpm_name("nginx".to_string());

    println!("  对象 ID: {}", object.id);
    println!("  对象版本: {}", object.ver);
    println!("  RPM 名称: {}", object.rpm_name);

    oval.add_rpm_info_object(object);
    println!("  添加后对象数量: {}", oval.get_object_count());
    println!();

    // 创建状态
    println!("【7. 创建 RPM 信息状态】");
    let evr = Evr::new()
        .with_datatype("evr_string".to_string())
        .with_operation("less than".to_string())
        .with_evr("0:1.20.1-1".to_string());

    println!("  EVR 数据类型: {}", evr.datatype);
    println!("  EVR 操作: {}", evr.operation);
    println!("  EVR 值: {}", evr.evr);

    let state = RpmInfoState::new()
        .with_id("oval:cn.chinaunicom.culinux.cusa:ste:20241001".to_string())
        .with_version("1".to_string())
        .with_evr(Some(evr));

    println!("  状态 ID: {}", state.id);
    println!("  状态版本: {}", state.version);
    println!("  包含 EVR: {}", state.evr.is_some());

    oval.add_rpminfo_state(state);
    println!("  添加后状态数量: {}", oval.get_state_count());
    println!();

    // 统计信息
    println!("【8. 最终统计信息】");
    println!("  定义数量: {}", oval.get_definition_count());
    println!("  测试数量: {}", oval.get_test_count());
    println!("  对象数量: {}", oval.get_object_count());
    println!("  状态数量: {}", oval.get_state_count());
    println!("  是否为空: {}", oval.is_empty());
    println!();

    // 生成 XML
    println!("【9. 生成 XML】");
    let xml = oval.to_oval_string()?;
    println!("  XML 长度: {} 字符", xml.len());
    println!(
        "  XML 包含定义ID: {}",
        xml.contains("oval:cn.chinaunicom.culinux.cusa:def:20241001")
    );
    println!("  XML 包含CVE: {}", xml.contains("CVE-2024-1234"));
    println!();

    // 保存到文件
    println!("【10. 保存到文件】");
    let output_path = "test/oval_demo_output.xml";
    match oval.save_to_file(output_path) {
        Ok(_) => println!("  成功保存到: {}", output_path),
        Err(e) => println!("  保存失败: {}", e),
    }
    println!();

    // 测试查找功能
    println!("【11. 测试查找功能】");
    let def_id = "oval:cn.chinaunicom.culinux.cusa:def:20241001";
    if let Some(found_def) = oval.definitions.find_by_id(def_id) {
        println!("  找到定义: {}", found_def.get_id());
        println!("  定义标题: {}", found_def.get_title());
    }

    let test_id = "oval:cn.chinaunicom.culinux.cusa:tst:20241001";
    if let Some(found_test) = oval.tests.find_by_id(test_id) {
        println!("  找到测试: {}", found_test.id);
        println!("  测试注释: {}", found_test.comment);
    }

    let state_id = "oval:cn.chinaunicom.culinux.cusa:ste:20241001";
    if let Some(found_state) = oval.states.find_by_id(state_id) {
        println!("  找到状态: {}", found_state.id);
    }
    println!();

    // 测试清空功能
    println!("【12. 测试清空功能】");
    println!("  清空前 - 定义数量: {}", oval.get_definition_count());
    oval.clear();
    println!("  清空后 - 定义数量: {}", oval.get_definition_count());
    println!("  清空后 - 是否为空: {}", oval.is_empty());
    println!();

    println!("示例执行完成！");
    Ok(())
}
