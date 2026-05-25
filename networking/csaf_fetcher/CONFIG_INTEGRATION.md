# CSAF Fetcher 配置集成更新

## 更新概述

成功集成 cu-scanner.toml 配置文件支持，使得 csaf_fetcher 可以从配置文件中读取 CSAF URL 并自动批量下载。

## 修改内容

### 1. utils/src/config.rs

#### 新增配置结构

```rust
/// CSAF URL配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CsafUrlConfig {
    /// index.txt文件的URL地址
    pub url: String,
}
```

#### 修改 AppConfig

```rust
pub struct AppConfig {
    pub database: DatabaseConfig,
    pub csaf_db: Option<DatabaseConfig>,
    pub csaf_url: Option<CsafUrlConfig>,  // 新增
    pub logging: LoggingConfig,
    pub api: ApiConfig,
}
```

#### 新增测试

- `test_csaf_url_config()` - 测试 csaf_url 配置的序列化和反序列化

**测试结果**: ✅ 所有 8 个测试通过

### 2. 配置文件格式 (cu-scanner.toml)

```toml
[csaf_url]
url = "https://dl-cdn.openeuler.openatom.cn/security/data/csaf/advisories/index.txt"
```

### 3. 新增示例程序

#### networking/csaf_fetcher/examples/fetch_with_config.rs

演示如何：
1. 从 cu-scanner.toml 加载配置
2. 提取 csaf_url 配置
3. 解析 index.txt URL 和基础 URL
4. 创建 CSAF 获取器
5. 批量下载 CSAF 文件

**代码行数**: 172 行

运行示例：
```bash
cargo run --example fetch_with_config
```

### 4. 依赖更新

在 `networking/csaf_fetcher/Cargo.toml` 中添加：

```toml
[dev-dependencies]
utils = { path = "../../utils" }
```

### 5. 文档更新

更新 `networking/csaf_fetcher/README.md`：
- 添加配置文件支持特性说明
- 添加使用配置文件的示例代码
- 添加配置文件格式说明
- 更新运行示例命令

## 使用方法

### 基本用法

```rust
use csaf_fetcher::{AsyncCsafFetcher, FetcherConfig};
use utils::config::AppConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. 加载配置文件
    let app_config = AppConfig::from_file("/path/to/cu-scanner.toml")?;

    // 2. 获取 CSAF URL 配置
    let csaf_url_config = app_config.csaf_url
        .ok_or("csaf_url 配置不存在")?;

    // 3. 解析 URL
    let index_url = &csaf_url_config.url;
    let base_url = &index_url[..index_url.rfind('/').unwrap()];

    // 4. 创建获取器
    let fetcher = AsyncCsafFetcher::with_defaults()?;

    // 5. 批量获取
    let results = fetcher.fetch_from_index_concurrent(
        index_url,
        base_url
    ).await?;

    println!("成功获取 {} 个 CSAF 文件", results.len());

    Ok(())
}
```

### 配置文件示例

```toml
# cu-scanner.toml

[database]
host = "localhost"
port = 5432
database = "cu_scanner"
username = "user"
password = "password"

[csaf_url]
url = "https://dl-cdn.openeuler.openatom.cn/security/data/csaf/advisories/index.txt"

[logging]
level = "info"
file = "/tmp/cu-scanner.log"
stdout = true

[api]
group_name = "api"
```

## 配置说明

### csaf_url 配置节

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| url | String | 是 | index.txt 文件的完整 URL 地址 |

### URL 解析规则

程序会自动从 index.txt URL 中提取基础 URL：

```
输入: https://example.com/security/data/csaf/advisories/index.txt
index_url: https://example.com/security/data/csaf/advisories/index.txt
base_url:  https://example.com/security/data/csaf/advisories
```

然后与 index.txt 中的相对路径拼接：
```
index.txt 内容: 2021/csaf-openeuler-sa-2021-1001.json
完整 URL: https://example.com/security/data/csaf/advisories/2021/csaf-openeuler-sa-2021-1001.json
```

## 集成优势

1. **统一配置管理** - 所有配置集中在 cu-scanner.toml 中
2. **灵活性** - 可以轻松切换不同的 CSAF 源
3. **可选配置** - csaf_url 为可选配置，不影响其他功能
4. **易于维护** - 修改 URL 无需重新编译代码

## 测试验证

### 运行单元测试

```bash
# 测试配置解析
cargo test -p utils test_csaf_url_config

# 测试所有 utils 功能
cargo test -p utils

# 测试 csaf_fetcher
cargo test -p csaf_fetcher
```

### 运行示例程序

```bash
# 使用配置文件批量获取
RUST_LOG=info cargo run --example fetch_with_config

# 查看详细日志
RUST_LOG=debug cargo run --example fetch_with_config
```

## 兼容性

- ✅ 完全向后兼容
- ✅ csaf_url 为可选配置
- ✅ 不影响现有功能
- ✅ 支持 Rust 2024 edition
- ✅ 支持 rustc 1.85.0+

## 后续扩展

可以考虑添加更多 csaf_url 配置选项：

```toml
[csaf_url]
url = "https://example.com/index.txt"
# 未来可扩展选项
# timeout = 60
# max_retries = 3
# cache_dir = "/var/cache/csaf"
# update_interval = 3600
```

## 总结

此次更新成功实现了：

1. ✅ 在 utils 中添加 CsafUrlConfig 配置结构
2. ✅ 集成到 AppConfig 中作为可选配置
3. ✅ 创建演示示例程序
4. ✅ 更新文档和测试
5. ✅ 所有测试通过
6. ✅ 保持向后兼容

现在 csaf_fetcher 可以方便地从 cu-scanner.toml 配置文件中读取 CSAF URL 并批量下载文件。
