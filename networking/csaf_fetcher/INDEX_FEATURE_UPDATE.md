# CSAF Fetcher - index.txt 功能更新

## 更新概述

成功为 csaf_fetcher 库添加了从 index.txt 文件解析并批量获取 CSAF 文件的功能。

## 新增功能

### 1. index.txt 文件解析

新增了解析 index.txt 文件的功能，该文件格式如下：

```
2021/csaf-openeuler-sa-2021-1001.json
2021/csaf-openeuler-sa-2021-1002.json
2021/csaf-openeuler-sa-2021-1003.json
2021/csaf-openeuler-sa-2021-1004.json
2022/csaf-openeuler-sa-2022-1001.json
```

每行包含一个 CSAF 文件的相对路径。

### 2. 同步 API (CsafFetcher)

#### 新增方法

1. **`fetch_index(index_url: &str)`**
   - 获取 index.txt 文件并解析出所有 CSAF 文件路径
   - 自动过滤空行和非 .json 文件
   - 返回文件路径列表

2. **`fetch_from_index(index_url: &str, base_url: &str)`**
   - 从 index.txt 文件批量获取所有 CSAF 文件
   - 自动拼接完整 URL：`{base_url}/{path}`
   - 返回每个文件的获取结果

3. **`fetch_from_index_and_save(index_url: &str, base_url: &str, output_dir: &str)`**
   - 从 index.txt 文件批量获取并保存所有 CSAF 文件
   - 自动创建输出目录
   - 文件名格式：将路径中的 `/` 替换为 `_`
   - 例如：`2021/csaf-xxx.json` → `2021_csaf-xxx.json`

### 3. 异步 API (AsyncCsafFetcher)

#### 新增方法

1. **`fetch_index(index_url: &str).await`**
   - 异步获取和解析 index.txt 文件

2. **`fetch_from_index(index_url: &str, base_url: &str).await`**
   - 顺序批量异步获取 CSAF 文件
   - 适合小规模或有速率限制的场景

3. **`fetch_from_index_concurrent(index_url: &str, base_url: &str).await`**
   - **并发批量异步获取 CSAF 文件**
   - 高性能，适合大规模下载
   - 所有文件同时下载

4. **`fetch_from_index_and_save(index_url: &str, base_url: &str, output_dir: &str).await`**
   - 异步批量获取并保存所有 CSAF 文件

## 使用示例

### 基本用法（同步）

```rust
use csaf_fetcher::CsafFetcher;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let fetcher = CsafFetcher::with_defaults()?;

    let index_url = "http://csaf-website/index.txt";
    let base_url = "http://csaf-website";

    // 批量获取
    let results = fetcher.fetch_from_index(index_url, base_url)?;

    for (path, result) in results {
        match result {
            Ok(csaf) => println!("✓ {}: {} 个漏洞", path, csaf.vulnerabilities.len()),
            Err(e) => println!("✗ {}: {}", path, e),
        }
    }

    Ok(())
}
```

### 批量获取并保存（同步）

```rust
use csaf_fetcher::CsafFetcher;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let fetcher = CsafFetcher::with_defaults()?;

    let results = fetcher.fetch_from_index_and_save(
        "http://csaf-website/index.txt",
        "http://csaf-website",
        "/tmp/csaf_files"
    )?;

    let success = results.iter().filter(|(_, r)| r.is_ok()).count();
    println!("成功保存 {} 个文件", success);

    Ok(())
}
```

### 并发批量获取（异步）

```rust
use csaf_fetcher::AsyncCsafFetcher;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let fetcher = AsyncCsafFetcher::with_defaults()?;

    let index_url = "http://csaf-website/index.txt";
    let base_url = "http://csaf-website";

    // 并发获取所有 CSAF 文件（高性能）
    let results = fetcher.fetch_from_index_concurrent(index_url, base_url).await?;

    let success = results.iter().filter(|(_, r)| r.is_ok()).count();
    println!("成功获取 {} / {} 个文件", success, results.len());

    Ok(())
}
```

## 工作流程

1. **下载 index.txt**
   ```
   GET http://csaf-website/index.txt
   ```

2. **解析文件路径**
   ```
   2021/csaf-openeuler-sa-2021-1001.json
   2021/csaf-openeuler-sa-2021-1002.json
   2021/csaf-openeuler-sa-2021-1003.json
   ```

3. **拼接完整 URL**
   ```
   http://csaf-website/2021/csaf-openeuler-sa-2021-1001.json
   http://csaf-website/2021/csaf-openeuler-sa-2021-1002.json
   http://csaf-website/2021/csaf-openeuler-sa-2021-1003.json
   ```

4. **批量下载**
   - 同步模式：逐个下载
   - 异步模式：可选顺序或并发下载

5. **保存文件（可选）**
   ```
   /tmp/csaf_files/2021_csaf-openeuler-sa-2021-1001.json
   /tmp/csaf_files/2021_csaf-openeuler-sa-2021-1002.json
   /tmp/csaf_files/2021_csaf-openeuler-sa-2021-1003.json
   ```

## 性能对比

### 顺序模式 vs 并发模式

假设下载 100 个文件，每个文件需要 1 秒：

| 模式 | 总耗时 | 说明 |
|------|--------|------|
| 同步顺序 | ~100 秒 | 逐个下载 |
| 异步顺序 | ~100 秒 | 异步但顺序执行 |
| 异步并发 | ~1-2 秒 | 同时下载所有文件 |

**建议：**
- 小规模（< 10 个文件）：使用同步或异步顺序模式
- 大规模（> 10 个文件）：使用异步并发模式
- 有速率限制：使用同步或异步顺序模式

## 新增测试

添加了 6 个新的单元测试：

1. `test_parse_index_content` - 测试 index.txt 内容解析
2. `test_url_construction` - 测试 URL 拼接
3. `test_url_construction_with_trailing_slash` - 测试 URL 末尾斜杠处理
4. `test_filename_conversion` - 测试文件名转换
5. `test_parse_index_with_empty_lines` - 测试空行过滤
6. `test_parse_index_filters_non_json` - 测试非 JSON 文件过滤

**总测试数**: 12 个（全部通过 ✅）

```
running 12 tests
test tests::test_fetcher_config_default ... ok
test tests::test_parse_index_content ... ok
test tests::test_parse_index_with_empty_lines ... ok
test tests::test_url_construction ... ok
test tests::test_parse_index_filters_non_json ... ok
test tests::test_url_construction_with_trailing_slash ... ok
test tests::test_filename_conversion ... ok
test tests::test_async_fetcher_creation ... ok
test tests::test_fetcher_creation ... ok
test tests::test_fetcher_with_defaults ... ok
test tests::test_async_fetcher_with_defaults ... ok
test tests::test_url_validation ... ok

test result: ok. 12 passed; 0 failed
```

## 新增示例程序

### 1. fetch_from_index.rs (同步版本)
- 演示从 index.txt 文件获取 CSAF 文件列表
- 演示批量获取 CSAF 文件
- 演示批量获取并保存到文件
- 151 行代码

### 2. async_fetch_from_index.rs (异步版本)
- 演示异步获取 index.txt
- 演示顺序批量异步获取
- 演示并发批量异步获取
- 演示性能对比
- 159 行代码

运行示例：
```bash
# 同步版本
cargo run --example fetch_from_index

# 异步版本
cargo run --example async_fetch_from_index
```

## 代码统计

### 主库代码
- **总行数**: 873 行（原 450 行 + 新增 423 行）
- **新增方法**: 8 个（同步 3 个 + 异步 5 个）
- **新增测试**: 6 个

### 示例代码
- **fetch_from_index.rs**: 151 行
- **async_fetch_from_index.rs**: 159 行
- **总计**: 310 行

## 文档更新

### README.md 更新
- 添加 index.txt 支持说明
- 添加 index.txt 文件格式说明
- 添加使用示例（同步和异步）
- 更新 API 文档
- 更新示例运行命令

### 新增文档
- **INDEX_FEATURE_UPDATE.md**: 本文档，详细说明 index.txt 功能

## 关键特性

1. ✅ **自动 URL 拼接** - 自动处理 base_url 末尾斜杠
2. ✅ **智能解析** - 自动过滤空行和非 .json 文件
3. ✅ **文件名转换** - 将路径转换为合法文件名
4. ✅ **批量处理** - 支持批量获取和保存
5. ✅ **并发支持** - 异步模式支持高性能并发下载
6. ✅ **错误处理** - 完善的错误处理和统计
7. ✅ **日志记录** - 详细的日志输出

## 实际应用场景

### 场景 1: 从官方源同步 CSAF 文件

```rust
let fetcher = AsyncCsafFetcher::with_defaults()?;

let results = fetcher.fetch_from_index_and_save(
    "https://www.openeuler.org/csaf/index.txt",
    "https://www.openeuler.org/csaf",
    "/var/lib/csaf/openeuler"
).await?;

println!("同步完成: {} 个文件", results.len());
```

### 场景 2: 定期更新 CSAF 数据库

```rust
use tokio::time::{interval, Duration};

let fetcher = AsyncCsafFetcher::with_defaults()?;
let mut timer = interval(Duration::from_secs(3600)); // 每小时

loop {
    timer.tick().await;

    match fetcher.fetch_from_index_concurrent(
        "https://example.com/index.txt",
        "https://example.com"
    ).await {
        Ok(results) => {
            let success = results.iter().filter(|(_, r)| r.is_ok()).count();
            println!("更新完成: {}/{} 成功", success, results.len());
        }
        Err(e) => eprintln!("更新失败: {}", e),
    }
}
```

### 场景 3: 镜像 CSAF 仓库

```rust
let fetcher = CsafFetcher::with_defaults()?;

let source_index = "https://source.example.com/index.txt";
let source_base = "https://source.example.com";
let mirror_dir = "/var/www/csaf-mirror";

fetcher.fetch_from_index_and_save(source_index, source_base, mirror_dir)?;
println!("镜像创建完成");
```

## 兼容性

- ✅ 完全向后兼容，不影响现有 API
- ✅ 支持 Rust 2024 edition
- ✅ 支持 rustc 1.85.0+
- ✅ 与现有 CSAF 数据结构完全兼容

## 未来改进

1. 支持增量更新（只下载新文件）
2. 支持断点续传
3. 支持并发数限制
4. 添加进度回调
5. 支持 index.txt 缓存
6. 支持文件校验（如 SHA256）

## 总结

成功为 csaf_fetcher 库添加了完整的 index.txt 支持，包括：

- ✅ 8 个新方法（同步 + 异步）
- ✅ 6 个新测试（全部通过）
- ✅ 2 个示例程序（310 行）
- ✅ 完整的文档更新
- ✅ 高性能并发支持
- ✅ 完全向后兼容

该功能使得从 index.txt 文件批量获取 CSAF 文件变得简单高效，特别是异步并发模式可以大幅提升大规模下载的性能。
