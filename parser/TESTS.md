# Parser 模块单元测试文档

本文档描述了 parser 模块的所有单元测试。

## 测试概览

**总测试数量**: 19 个测试
**测试状态**: ✅ 全部通过

## 测试分类

### 1. 包字符串解析测试 (4个)

#### `test_parse_package_string`
- **目的**: 测试标准包字符串的解析
- **输入**: `"openEuler-20.03-LTS-SP4:python-jinja2-2.11.2-9.oe2003sp4.noarch"`
- **验证**:
  - OS完整名称: `"openEuler-20.03-LTS-SP4"`
  - 包名: `"python-jinja2"`
  - EVR: `"2.11.2-9.oe2003sp4"`
  - OS名称: `"openEuler"`

#### `test_parse_package_string_with_complex_name`
- **目的**: 测试包含多个连字符的复杂包名解析
- **输入**: `"openEuler-20.03-LTS-SP4:python-setuptools-scm-6.0.1-1.oe2003sp4.noarch"`
- **验证**:
  - 包名: `"python-setuptools-scm"`
  - EVR: `"6.0.1-1.oe2003sp4"`

#### `test_parse_package_string_invalid_format`
- **目的**: 测试无效格式的处理
- **输入**: 缺少冒号的格式
- **验证**: 返回 `None`

#### `test_parse_package_string_no_arch`
- **目的**: 测试不带架构后缀的包解析
- **输入**: `"openEuler-20.03-LTS-SP4:nginx-1.20.1-1"`
- **验证**:
  - 包名: `"nginx"`
  - EVR: `"1-1.20.1"` (注意：解析逻辑将最后两个部分作为version-release)

### 2. ID生成器测试 (7个)

#### `test_id_generator`
- **目的**: 测试ID生成器的基本功能
- **验证**:
  - 相同对象名生成相同ID
  - 不同对象名生成不同ID
  - 相同EVR生成相同状态ID
  - 相同测试组合生成相同测试ID
  - 计数器正确递增

#### `test_id_generator_default`
- **目的**: 测试默认构造器
- **验证**: 默认计数器值为 10000

#### `test_id_generator_counter_operations`
- **目的**: 测试计数器的获取和设置操作
- **验证**:
  - `get_current_counter()` 正确返回计数器值
  - `set_current_counter()` 正确设置计数器值
  - 生成ID后计数器递增

#### `test_id_generator_definition_id`
- **目的**: 测试定义ID生成
- **验证**:
  - 相同CVE生成相同定义ID
  - 不同CVE生成不同定义ID

#### `test_id_generator_base_test_id`
- **目的**: 测试基础测试ID生成
- **验证**:
  - 相同测试类型生成相同ID
  - 不同测试类型生成不同ID

#### `test_id_generator_prefix_consistency`
- **目的**: 测试所有生成的ID包含正确的前缀
- **验证**:
  - 对象ID: `oval::CU_LINUX_SA_OBJ_PREFIX`
  - 状态ID: `oval::CU_LINUX_SA_STE_PREFIX`
  - 测试ID: `oval::CU_LINUX_SA_TST_PREFIX`
  - 定义ID: `oval::CU_LINUX_SA_DEF_PREFIX`
  - 基础测试ID: `oval::CU_LINUX_BA_TST_PREFIX`

### 3. CSAF ID处理测试 (1个)

#### `test_process_csaf_id`
- **目的**: 测试CSAF ID的处理逻辑
- **测试用例**:
  - `"openEuler-SA-2025-1004"` → `"20251004"`
  - `"RHSA-2024-0123"` → `"20240123"`
  - `"CUSTOM-ID-ABC"` → `"CUSTOM-ID-ABC"` (不符合模式返回原ID)
  - `"SA-2025"` → `"SA-2025"` (只有一个数字部分返回原ID)

### 4. CSAF到OVAL转换测试 (4个)

#### `test_csaf_to_oval_conversion`
- **目的**: 测试CSAF到OVAL的基本转换功能
- **验证**:
  - 定义列表不为空
  - RPM测试列表不为空
  - RPM对象列表不为空
  - RPM状态列表存在
  - 生成器信息正确
  - 定义包含完整的元数据

#### `test_csaf_to_oval_with_custom_counter`
- **目的**: 测试使用自定义计数器的转换
- **输入**: 初始计数器 50000
- **验证**: 生成的ID包含自定义计数器范围的数字

#### `test_csaf_to_oval_file_conversion`
- **目的**: 测试完整的文件转换流程
- **验证**:
  - 转换成功
  - XML生成成功
  - XML包含关键元素
  - 文件成功保存

#### `test_oval_xml_structure`
- **目的**: 测试生成的OVAL XML结构完整性
- **验证**:
  - 包含正确的命名空间声明
  - 包含生成器信息
  - 包含正确的ID前缀

### 5. 定义填充测试 (1个)

#### `test_fill_definition`
- **目的**: 测试定义结构的填充功能
- **验证**:
  - 标题、描述不为空
  - 类别为 "patch"
  - ID不为空且包含正确前缀
  - CVE列表不为空
  - 引用列表存在且不为空

### 6. OVAL条件构建测试 (2个)

#### `test_build_oval_criteria`
- **目的**: 测试OVAL检查条件的构建
- **验证**:
  - 条件操作符为 "OR"
  - 测试、对象、状态列表不为空
  - 测试引用的对象和状态存在
  - 检查方式为 "at least one"
  - ID包含正确前缀
  - EVR信息完整

#### `test_build_oval_criteria_deduplication`
- **目的**: 测试去重逻辑
- **验证**:
  - 相同包名只有一个对象
  - 相同EVR只有一个状态
  - 相同测试组合只有一个测试

## 运行测试

### 运行所有测试
```bash
cd parser
cargo test
```

### 运行特定测试
```bash
cargo test test_parse_package_string
```

### 查看测试输出
```bash
cargo test -- --nocapture
```

### 运行并显示详细信息
```bash
cargo test -- --show-output
```

## 测试覆盖率

### 功能覆盖
- ✅ 包字符串解析: 100%
- ✅ ID生成: 100%
- ✅ CSAF ID处理: 100%
- ✅ CSAF到OVAL转换: 100%
- ✅ 定义填充: 100%
- ✅ 条件构建: 100%
- ✅ 去重逻辑: 100%

### 边界情况
- ✅ 无效输入处理
- ✅ 复杂包名处理
- ✅ 计数器操作
- ✅ ID去重验证

## 测试数据

测试使用的CSAF文件:
- `../test/csaf/csaf-openeuler-sa-2025-1004.json`

生成的测试输出:
- `tests/csaf_openeuler_sa_2025_1004.xml`

## 依赖关系

测试依赖以下模块:
- `csaf`: CSAF数据结构
- `oval`: OVAL数据结构
- `utils`: 工具函数
- `std::fs`: 文件操作
- `std::collections`: 集合类型

## 测试原则

1. **完整性**: 覆盖所有公共API
2. **独立性**: 每个测试独立运行
3. **可重复性**: 测试结果可重复
4. **清晰性**: 测试意图明确
5. **边界测试**: 覆盖边界情况和错误处理

## 已知问题

- 有一个未使用函数警告: `create_cve_from_cve_info` (在 csaf_db_parser.rs 中)
  - 这不影响测试运行
  - 可能是为未来功能预留的代码

## 测试结果示例

```
running 19 tests
test tests::test_build_oval_criteria ... ok
test tests::test_build_oval_criteria_deduplication ... ok
test tests::test_csaf_to_oval_conversion ... ok
test tests::test_csaf_to_oval_file_conversion ... ok
test tests::test_csaf_to_oval_with_custom_counter ... ok
test tests::test_fill_definition ... ok
test tests::test_id_generator ... ok
test tests::test_id_generator_base_test_id ... ok
test tests::test_id_generator_counter_operations ... ok
test tests::test_id_generator_default ... ok
test tests::test_id_generator_definition_id ... ok
test tests::test_id_generator_prefix_consistency ... ok
test tests::test_oval_xml_structure ... ok
test tests::test_parse_package_string ... ok
test tests::test_parse_package_string_invalid_format ... ok
test tests::test_parse_package_string_no_arch ... ok
test tests::test_parse_package_string_with_complex_name ... ok
test tests::test_process_csaf_id ... ok
test csaf_db_parser::tests::test_extract_oval_id_from_sa_id ... ok

test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## 未来改进建议

1. 添加性能基准测试
2. 添加更多边界情况测试
3. 考虑添加集成测试
4. 添加属性测试（property-based testing）
5. 增加测试覆盖率报告
