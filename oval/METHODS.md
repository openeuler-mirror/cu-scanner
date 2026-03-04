# OVAL 数据结构方法文档

本文档列出了为 OVAL 数据结构添加的所有便捷方法和自测代码。

## OvalDefinitions (OVAL 定义根结构)

### 构造方法
- `new()` - 创建新的 OvalDefinitions 实例

### 添加方法
- `add_definition(definition)` - 添加定义
- `add_rpminfo_test(test)` - 添加 RPM 信息测试
- `add_rpm_info_object(object)` - 添加 RPM 信息对象
- `add_rpminfo_state(state)` - 添加 RPM 信息状态

### 获取方法
- `get_definition_count()` - 获取定义数量
- `get_test_count()` - 获取测试数量
- `get_object_count()` - 获取对象数量
- `get_state_count()` - 获取状态数量

### 检查方法
- `is_empty()` - 检查是否为空（没有任何定义）

### 操作方法
- `clear()` - 清空所有内容

### I/O 方法
- `to_oval_string()` - 将 OVAL 定义转换为 XML 字符串
- `save_to_file(path)` - 保存到文件

## Definitions (定义列表)

### 构造方法
- `new()` - 创建新的 Definitions 实例

### 操作方法
- `add(definition)` - 添加定义
- `len()` - 获取定义数量
- `is_empty()` - 检查是否为空
- `find_by_id(id)` - 根据 ID 查找定义

## Definition (定义)

### 构造方法
- `new()` - 创建新的 Definition 实例

### 构建器方法
- `with_id(id)` - 设置 ID
- `with_class(class)` - 设置类别
- `with_version(version)` - 设置版本
- `with_metadata(metadata)` - 设置元数据
- `with_criteria(criteria)` - 设置条件

### 获取方法
- `get_id()` - 获取 ID
- `get_title()` - 获取标题

## Metadata (元数据)

### 构造方法
- `new()` - 创建新的 Metadata 实例

### 操作方法
- `add_reference(reference)` - 添加引用
- `get_reference_count()` - 获取引用数量

## Advisory (建议信息)

### 构造方法
- `new()` - 创建新的 Advisory 实例

### 操作方法
- `add_cve(cve)` - 添加 CVE
- `get_cve_count()` - 获取 CVE 数量
- `get_cve_ids()` - 获取所有 CVE ID
- `contains_cve(cve_id)` - 检查是否包含指定 CVE

## CVE (CVE 信息)

### 构造方法
- `new()` - 创建新的 CVE 实例

### 构建器方法
- `with_content(content)` - 设置内容（CVE ID）
- `with_cvss3(cvss3)` - 设置 CVSS3 评分
- `with_href(href)` - 设置链接
- `with_impact(impact)` - 设置影响程度

### 获取方法
- `get_id()` - 获取 CVE ID

## Criteria (检查条件)

### 构造方法
- `new()` - 创建新的 Criteria 实例

### 操作方法
- `add_criterion(criterion)` - 添加条件
- `add_sub_criteria(criteria)` - 添加子条件
- `get_criterion_count()` - 获取条件数量
- `get_sub_criteria_count()` - 获取子条件数量

### 构建器方法
- `with_operator(operator)` - 设置操作符

## Tests (测试列表)

### 构造方法
- `new()` - 创建新的 Tests 实例

### 操作方法
- `add_rpminfo_test(test)` - 添加 RPM 信息测试
- `len()` - 获取测试数量
- `is_empty()` - 检查是否为空
- `find_by_id(id)` - 根据 ID 查找测试

## RpmInfoTest (RPM 信息测试)

### 构造方法
- `new()` - 创建新的 RpmInfoTest 实例

### 构建器方法
- `with_id(id)` - 设置 ID
- `with_check(check)` - 设置检查方式
- `with_comment(comment)` - 设置注释
- `with_version(version)` - 设置版本
- `with_object_ref(object_ref)` - 设置对象引用
- `with_state_ref(state_ref)` - 设置状态引用

## Objects (对象列表)

### 构造方法
- `new()` - 创建新的 Objects 实例

### 操作方法
- `add_rpm_info(object)` - 添加 RPM 信息对象
- `clear()` - 清空所有对象
- `is_empty()` - 检查是否为空
- `len()` - 获取对象总数
- `has_rpm_info_objects()` - 检查是否包含 RPM 信息对象
- `rpm_info_count()` - 获取 RPM 信息对象数量

### 迭代器方法
- `iter_rpm_info()` - 获取 RPM 信息对象的只读迭代器
- `iter_mut_rpm_info()` - 获取 RPM 信息对象的可变迭代器

## RpmInfoObject (RPM 信息对象)

### 构造方法
- `new()` - 创建新的 RpmInfoObject 实例

### 构建器方法
- `with_id(id)` - 设置 ID
- `with_ver(ver)` - 设置版本
- `with_rpm_name(rpm_name)` - 设置 RPM 名称

## States (状态列表)

### 构造方法
- `new()` - 创建新的 States 实例

### 操作方法
- `add_rpminfo_state(state)` - 添加 RPM 信息状态
- `len()` - 获取状态数量
- `is_empty()` - 检查是否为空
- `find_by_id(id)` - 根据 ID 查找状态

## RpmInfoState (RPM 信息状态)

### 构造方法
- `new()` - 创建新的 RpmInfoState 实例

### 构建器方法
- `with_id(id)` - 设置 ID
- `with_version(version)` - 设置版本
- `with_evr(evr)` - 设置 EVR 信息

## Evr (EVR 信息)

### 构造方法
- `new()` - 创建新的 Evr 实例

### 构建器方法
- `with_datatype(datatype)` - 设置数据类型
- `with_operation(operation)` - 设置操作方式
- `with_evr(evr)` - 设置 EVR 值

## 使用示例

```rust
use oval::*;

fn main() -> Result<()> {
    // 创建 OVAL 定义
    let mut oval = OvalDefinitions::new();
    oval.generator.time_stamp = "2024-01-01T12:00:00".to_string();

    // 创建元数据和 CVE
    let mut metadata = Metadata::new();
    metadata.title = "安全更新".to_string();

    let cve = CVE::new()
        .with_content("CVE-2024-1234".to_string())
        .with_cvss3("7.5".to_string())
        .with_impact("High".to_string());

    metadata.advisory.add_cve(cve);

    // 创建定义
    let definition = Definition::new()
        .with_id("oval:test:def:1".to_string())
        .with_class("patch".to_string())
        .with_version(1)
        .with_metadata(metadata);

    oval.add_definition(definition);

    // 创建测试
    let test = RpmInfoTest::new()
        .with_id("oval:test:tst:1".to_string())
        .with_check("all".to_string())
        .with_comment("检查软件包版本".to_string())
        .with_object_ref("oval:test:obj:1".to_string())
        .with_state_ref("oval:test:ste:1".to_string());

    oval.add_rpminfo_test(test);

    // 创建对象
    let object = RpmInfoObject::new()
        .with_id("oval:test:obj:1".to_string())
        .with_rpm_name("nginx".to_string());

    oval.add_rpm_info_object(object);

    // 创建状态
    let evr = Evr::new()
        .with_datatype("evr_string".to_string())
        .with_operation("less than".to_string())
        .with_evr("0:1.20.1-1".to_string());

    let state = RpmInfoState::new()
        .with_id("oval:test:ste:1".to_string())
        .with_version("1".to_string())
        .with_evr(Some(evr));

    oval.add_rpminfo_state(state);

    // 统计信息
    println!("定义数量: {}", oval.get_definition_count());
    println!("测试数量: {}", oval.get_test_count());
    println!("对象数量: {}", oval.get_object_count());
    println!("状态数量: {}", oval.get_state_count());

    // 生成 XML 并保存
    let xml = oval.to_oval_string()?;
    oval.save_to_file("output.xml")?;

    Ok(())
}
```

## 自测代码

项目包含了全面的单元测试，覆盖所有新增方法：

### 测试列表

1. `test_oval_definitions_basic_operations` - 测试 OvalDefinitions 基本操作
2. `test_definition_builder` - 测试 Definition 构建器模式
3. `test_definitions_operations` - 测试 Definitions 操作
4. `test_metadata_references` - 测试 Metadata 引用操作
5. `test_advisory_cve_operations` - 测试 Advisory CVE 操作
6. `test_cve_builder` - 测试 CVE 构建器
7. `test_criteria_operations` - 测试 Criteria 操作
8. `test_tests_operations` - 测试 Tests 操作
9. `test_rpminfo_test_builder` - 测试 RpmInfoTest 构建器
10. `test_objects_operations` - 测试 Objects 操作
11. `test_rpminfo_object_builder` - 测试 RpmInfoObject 构建器
12. `test_states_operations` - 测试 States 操作
13. `test_rpminfo_state_builder` - 测试 RpmInfoState 构建器
14. `test_evr_builder` - 测试 Evr 构建器
15. `test_complete_oval_workflow` - 测试完整的 OVAL 工作流程

### 运行测试

```bash
# 运行所有测试
cargo test

# 运行特定测试
cargo test test_oval_definitions_basic_operations

# 查看测试输出
cargo test -- --nocapture
```

### 运行示例程序

```bash
cargo run --example oval_methods_demo
```

## 设计特点

1. **构建器模式**：大多数结构体都提供了 `with_*` 方法，支持链式调用，使代码更简洁
2. **一致性**：所有方法命名遵循 Rust 命名规范
3. **完整性**：提供了添加、获取、查找、检查等全套操作
4. **安全性**：使用 `Option` 处理可能为空的值
5. **便捷性**：提供了高级操作如 `clear()`, `is_empty()`, `find_by_id()` 等

## 测试覆盖

- ✅ 所有新增方法都有对应的单元测试
- ✅ 测试覆盖正常流程和边界情况
- ✅ 包含完整的工作流程测试
- ✅ 16 个测试全部通过

这些方法使得 OVAL 数据结构更加易用，符合 Rust 的最佳实践。
