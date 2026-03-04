# CSAF 数据结构方法文档

本文档列出了为 CSAF 数据结构添加的所有便捷方法。

## CSAF 主结构

### 构造方法
- `new()` - 创建新的空 CSAF 实例

### 获取方法
- `get_id()` - 获取 CSAF 文档的 ID
- `get_version()` - 获取 CSAF 文档的版本
- `get_title()` - 获取 CSAF 文档的标题
- `get_vulnerability_count()` - 获取漏洞数量
- `get_cve_ids()` - 获取所有 CVE ID 列表
- `get_status()` - 获取文档状态
- `get_release_date()` - 获取发布日期
- `get_initial_release_date()` - 获取初始发布日期

### 检查方法
- `contains_cve(&str)` - 检查是否包含指定的 CVE

### I/O 方法
- `from_file(&str)` - 从文件路径加载 CSAF 数据
- `to_file(&str)` - 将 CSAF 数据保存到文件
- `from_url(&str)` - 从 URL 加载 CSAF 数据

## Document

### 构造方法
- `new()` - 创建新的空 Document 实例

### 获取方法
- `get_category()` - 获取文档类别
- `get_lang()` - 获取文档语言
- `get_publisher_name()` - 获取发布者名称

## AggregateSeverity (严重性)

### 构造方法
- `new()` - 创建新的空 AggregateSeverity 实例

### 获取方法
- `get_severity()` - 获取严重性级别

### 检查方法
- `is_critical()` - 检查是否为严重级别
- `is_high()` - 检查是否为高危级别

## Distribution

### 构造方法
- `new()` - 创建新的 Distribution 实例

## Tlp (交通灯协议)

### 构造方法
- `new()` - 创建新的 Tlp 实例，默认为 WHITE

### 检查方法
- `is_public()` - 检查是否可以公开共享

## Publisher

### 构造方法
- `new()` - 创建新的 Publisher 实例

## Tracking (跟踪信息)

### 构造方法
- `new()` - 创建新的 Tracking 实例

### 获取方法
- `get_revision_count()` - 获取修订历史数量
- `get_latest_revision()` - 获取最新修订信息

## Generator

### 构造方法
- `new()` - 创建新的 Generator 实例

## Engine

### 构造方法
- `new()` - 创建新的 Engine 实例

## ProductTree (产品树)

### 构造方法
- `new()` - 创建新的 ProductTree 实例

### 获取方法
- `get_all_product_ids()` - 获取所有产品 ID 列表
- `get_product_count()` - 获取产品数量

## Vulnerabilitie (漏洞)

### 构造方法
- `new()` - 创建新的 Vulnerabilitie 实例

### 获取方法
- `get_cve_id()` - 获取 CVE ID
- `get_title()` - 获取标题
- `get_affected_product_count()` - 获取受影响的产品数量
- `get_cvss_score()` - 获取 CVSS 分数（如果有）
- `get_severity()` - 获取严重性级别（如果有）

### 检查方法
- `is_critical()` - 检查是否为严重漏洞
- `is_high()` - 检查是否为高危漏洞

## ProductStatus (产品状态)

### 构造方法
- `new()` - 创建新的 ProductStatus 实例

### 获取方法
- `get_fixed_products()` - 获取已修复的产品列表

### 检查方法
- `is_product_fixed(&str)` - 检查产品是否已修复

## Score (评分)

### 获取方法
- `get_base_score()` - 获取 CVSS 基础分数
- `get_severity()` - 获取严重性级别
- `get_vector_string()` - 获取向量字符串

## CvssV3

### 构造方法
- `new()` - 创建新的 CvssV3 实例

### 检查方法
- `is_critical()` - 检查是否为严重级别（分数 >= 9.0 或标签为 "critical"）
- `is_high()` - 检查是否为高危级别（分数 >= 7.0 且 < 9.0 或标签为 "high"）
- `is_medium()` - 检查是否为中危级别（分数 >= 4.0 且 < 7.0 或标签为 "medium"）
- `is_low()` - 检查是否为低危级别（分数 > 0.0 且 < 4.0 或标签为 "low"）

## 使用示例

```rust
use csaf::CSAF;

fn main() -> utils::Result<()> {
    // 加载 CSAF 数据
    let csaf = CSAF::from_file("path/to/csaf.json")?;

    // 获取基本信息
    println!("文档 ID: {}", csaf.get_id());
    println!("标题: {}", csaf.get_title());
    println!("漏洞数量: {}", csaf.get_vulnerability_count());

    // 获取所有 CVE ID
    let cve_ids = csaf.get_cve_ids();
    for cve_id in cve_ids {
        println!("CVE: {}", cve_id);
    }

    // 检查严重性
    if csaf.document.aggregate_severity.is_critical() {
        println!("这是一个严重的安全公告！");
    }

    // 检查漏洞
    for vuln in &csaf.vulnerabilities {
        if vuln.is_critical() || vuln.is_high() {
            println!("高危漏洞: {} - {}", vuln.get_cve_id(), vuln.get_title());

            if let Some(score) = vuln.get_cvss_score() {
                println!("  CVSS 分数: {}", score);
            }
        }
    }

    // 检查产品修复状态
    if let Some(vuln) = csaf.vulnerabilities.first() {
        for product_id in csaf.product_tree.get_all_product_ids() {
            if vuln.product_status.is_product_fixed(&product_id) {
                println!("产品 {} 已修复", product_id);
            }
        }
    }

    Ok(())
}
```

## 运行示例程序

```bash
cargo run --example csaf_methods_demo
```

这个示例程序展示了所有方法的使用。
