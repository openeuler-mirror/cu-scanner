# cu-scanner

cu-scanner是一个安全漏洞扫描与分析工具，用于处理CSAF(Common Security Advisory Framework)格式的安全公告，并将其转换为OVAL(Open Vulnerability and Assessment Language)格式。

## 功能特性

- 读取CSAF格式的安全公告文件
- 将CSAF格式转换为OVAL格式
- 支持将转换结果保存到文件或输出到标准输出
- 数据库存储OVAL定义及相关信息
- 命令行界面，易于使用

## 安装

确保系统已安装Rust开发环境，然后克隆项目并构建：

```bash
git clone <repository-url>
cd cu-scanner
cargo build --release
```

## 使用方法

### 基本用法

```bash
# 将CSAF文件转换为OVAL格式并保存到指定文件
cargo run --bin cu-scanner -- -f <csaf-file> -o <output-file>

# 将CSAF文件转换为OVAL格式并输出到标准输出
cargo run --bin cu-scanner -- -f <csaf-file>
```

### 命令行参数

- `-f, --csaf-file <csaf-file>`: 指定要处理的单个CSAF文件路径
- `-D, --csaf-dir <csaf-dir>`: 指定CSAF文件目录路径，处理目录中的所有CSAF文件
- `-F, --fetch-csaf`: 从网络获取CSAF文件（使用配置文件中的csaf_url）
- `-o, --output <output-file>`: 指定转换结果的输出文件路径（可选，仅用于单文件处理）
- `-c, --config <config-file>`: 指定配置文件路径（默认为/etc/cu-scanner/cu-scanner.toml）
- `-d, --daemon`: 以守护进程方式运行服务

### 示例

#### 单文件处理

```bash
# 处理CSAF文件并将结果保存到output.xml
cargo run --bin cu-scanner -- -f test/csaf/csaf-openeuler-sa-2025-1004.json -o output.xml

# 处理CSAF文件并输出到标准输出
cargo run --bin cu-scanner -- -f test/csaf/csaf-openeuler-sa-2025-1004.json
```

#### 批量处理目录中的CSAF文件

```bash
# 处理目录中的所有CSAF文件，自动检查数据库并跳过已存在的定义
cargo run --bin cu-scanner -- -D test/csaf/

# 使用自定义配置文件
cargo run --bin cu-scanner -- -c /path/to/config.toml -D /path/to/csaf/files/
```

**批量处理功能说明：**

当使用 `-D` 参数指定目录时，程序会：

1. 扫描目录中的所有 `.json` 文件
2. 从文件名中提取OVAL ID（例如：`csaf-openeuler-sa-2025-1004.json` -> `oval:org.openeuler.cu-scanner:def:20251004`）
3. 在数据库中查询该OVAL ID是否已存在
4. 如果已存在，跳过该文件
5. 如果不存在，读取CSAF文件，转换为OVAL格式，并保存到数据库

这种方式可以高效地批量导入CSAF文件，避免重复处理已经存在的定义。

#### 从网络获取CSAF文件

```bash
# 从网络获取CSAF文件（使用配置文件中的csaf_url）
cargo run --bin cu-scanner -- -F

# 使用自定义配置文件
cargo run --bin cu-scanner -- -c /path/to/config.toml -F
```

**网络获取功能说明：**

当使用 `-F` 参数时，程序会：

1. 从配置文件中读取 `csaf_url.url` 配置（通常是index.txt的URL）
2. 从 index.txt 文件中解析出所有CSAF文件的相对路径
3. 对于每个文件：
   - 从文件名提取OVAL ID
   - 查询数据库检查是否已存在
   - 如果不存在，从网络下载文件
   - 转换为OVAL格式并保存到数据库
4. 自动跳过已存在的OVAL定义，避免重复处理

**配置文件示例：**

```toml
[csaf_url]
url = "http://example.com/csaf/advisories/index.txt"
# 定时获取间隔（秒），默认为3600秒（1小时）
fetch_interval_secs = 3600
```

这种方式适用于自动化场景，可以定期从远程服务器同步最新的CSAF文件。`fetch_interval_secs`参数用于在daemon模式下控制定时获取的时间间隔。

## 项目结构

- `csaf/`: CSAF数据结构定义和处理模块
- `database/`: 数据库相关操作模块
- `oval/`: OVAL数据结构定义和处理模块
- `parser/`: CSAF到OVAL格式转换模块
- `utils/`: 工具模块，包括配置文件处理和日志系统
- `cu-scanner/`: 主程序入口

## 依赖

- Rust 1.56或更高版本
- PostgreSQL数据库（用于存储OVAL定义）
- 其他Rust crate依赖（详见各模块的Cargo.toml文件）

