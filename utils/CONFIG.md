# Utils Crate - 配置管理

该 crate 提供了 cu-scanner 项目的配置管理功能。

## 配置文件结构

cu-scanner 使用 TOML 格式的配置文件，默认路径为 `config/cu-scanner.toml`。

### 完整配置示例

```toml
# cu-scanner 配置文件

[database]
# 主数据库配置（用于存储 OVAL 定义等数据）
host = "localhost"
port = 5432
database = "cu_scanner"
username = "cu_scanner"
password = "your_password"

[csaf_db]
# CSAF 数据库配置（用于读取 CSAF 安全公告数据）
host = "localhost"
port = 5432
database = "cu_cveadmin"
username = "cu_cveadmin"
password = "your_password"

[logging]
# 日志配置
level = "info"  # 可选值: trace, debug, info, warn, error
file = "/tmp/cu-scanner.log"  # 日志文件路径
stdout = true  # 是否输出到标准输出

[api]
# API 配置
group_name = "api"  # API 分组名称
```

## 配置字段说明

### [database] - 主数据库配置
用于存储 OVAL 定义、测试、对象、状态等数据的数据库配置。

- `host`: 数据库主机地址
- `port`: 数据库端口号
- `database`: 数据库名称
- `username`: 数据库用户名
- `password`: 数据库密码

### [csaf_db] - CSAF 数据库配置（可选）
用于读取 CSAF 安全公告数据的数据库配置。如果不配置此节，则不会从 CSAF 数据库读取数据。

- `host`: CSAF 数据库主机地址
- `port`: CSAF 数据库端口号
- `database`: CSAF 数据库名称
- `username`: CSAF 数据库用户名
- `password`: CSAF 数据库密码

### [logging] - 日志配置
配置应用程序的日志行为。

- `level`: 日志级别（trace, debug, info, warn, error）
- `file`: 日志文件路径（当 stdout 为 false 时使用）
- `stdout`: 是否输出到标准输出（默认 false）

**注意**：当 `stdout = true` 时，日志会输出到标准输出，忽略 `file` 配置。

### [api] - API 配置
配置 API 相关选项。

- `group_name`: API 分组名称（默认 "api"）

## 使用方法

### 在代码中加载配置

```rust
use utils::config::AppConfig;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 从文件加载配置
    let config = AppConfig::from_file("config/cu-scanner.toml")?;

    // 访问主数据库配置
    println!("数据库主机: {}", config.database.host);

    // 访问 CSAF 数据库配置（如果有配置）
    if let Some(ref csaf_db) = config.csaf_db {
        println!("CSAF 数据库: {}", csaf_db.database);
    }

    // 访问日志配置
    println!("日志级别: {}", config.logging.level);

    Ok(())
}
```

### 保存配置到文件

```rust
use utils::config::{AppConfig, DatabaseConfig, LoggingConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = AppConfig {
        database: DatabaseConfig {
            host: "localhost".to_string(),
            port: 5432,
            database: "cu_scanner".to_string(),
            username: "user".to_string(),
            password: "password".to_string(),
        },
        csaf_db: Some(DatabaseConfig {
            host: "localhost".to_string(),
            port: 5432,
            database: "cu_cveadmin".to_string(),
            username: "csaf_user".to_string(),
            password: "csaf_password".to_string(),
        }),
        logging: LoggingConfig {
            level: "info".to_string(),
            file: "/tmp/cu-scanner.log".to_string(),
            stdout: true,
        },
        ..Default::default()
    };

    // 保存到文件
    config.save_to_file("config/cu-scanner.toml")?;

    Ok(())
}
```

## 运行示例

```bash
# 运行配置加载示例
cargo run --example load_config_demo
```

## 测试

```bash
# 运行所有配置相关的测试
cargo test --lib config

# 运行特定测试
cargo test --lib test_csaf_db_config
```

## 配置验证

在启动应用程序时，建议验证配置的有效性：

```rust
use utils::config::AppConfig;

fn validate_config(config: &AppConfig) -> Result<(), String> {
    // 验证数据库配置
    if config.database.host.is_empty() {
        return Err("数据库主机不能为空".to_string());
    }

    // 验证日志级别
    let valid_levels = ["trace", "debug", "info", "warn", "error"];
    if !valid_levels.contains(&config.logging.level.as_str()) {
        return Err(format!("无效的日志级别: {}", config.logging.level));
    }

    Ok(())
}
```
