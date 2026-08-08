# cu-scanner 设计文档

## 1. 概述

cu-scanner 是一个基于 Rust 开发的工具/服务，核心功能是将 CSAF（Common Security Advisory Framework）格式的 JSON 安全公告文件转换为 OVAL（Open Vulnerability and Assessment Language）标准的 XML 补丁定义文件，同时提供命令行工具和 HTTP API 两种使用方式。

### 1.1 核心功能
- **CSAF → OVAL 转换**：将单个或多个 CSAF JSON 文件转换为符合 OVAL 规范的 XML 文件。
- **远程同步下载**：从远程 Web 源或 `index.txt` 文件列表自动下载 CSAF 文件，仅同步新增/变更内容。
- **数据持久化**：将 OVAL 元数据持久化到数据库，支持 MySQL / SQLite / PostgreSQL。
- **HTTP API**：对外提供 OVAL 文件查询接口，支持按名称、月份、时间段获取 OVAL XML。
- **合并输出**：支持将多个 OVAL 定义合并为单个符合规范的 XML 文件返回。

### 1.2 运行模式
| 模式 | 说明 |
|------|------|
| 命令行模式 (CLI) | 单次转换（单文件/目录），不依赖数据库和 HTTP 服务 |
| 服务模式 (Server) | 启动后台服务，提供下载同步、数据库存储、HTTP API 能力 |

---

## 2. 术语定义

| 术语 | 说明 |
|------|------|
| **CSAF** | Common Security Advisory Framework，通用安全公告框架，JSON 格式标准（如 `csaf-cuos-sa-2025-1665.json`）。 |
| **OVAL** | Open Vulnerability and Assessment Language，开放漏洞与评估语言，XML 格式标准，用于定义系统配置检测逻辑。 |
| **CVE** | Common Vulnerabilities and Exposures，通用漏洞披露编号。 |
| **CPE** | Common Platform Enumeration，通用平台枚举，用于标识软硬件产品。 |
| **Definition** | OVAL 中的顶层定义单元，对应一个安全公告（patch/advisory），包含元数据和检测条件（criteria）。 |
| **Test** | OVAL 中的检测测试，如 `rpminfo_test` 检测 RPM 包信息。 |
| **Object** | OVAL 中 Test 的操作对象，如 `rpminfo_object` 指定要检测的 RPM 包名。 |
| **State** | OVAL 中 Test 的期望状态，如 `rpminfo_state` 指定版本小于某值。 |
| **Criteria** | OVAL Definition 中的逻辑判断条件，支持 `AND` / `OR` 组合。 |
| **EVR** | Epoch:Version-Release，RPM 包的版本字符串格式（如 `0:9.6p1-6.ule4`）。 |
| **index.txt** | 远程 CSAF 文件索引列表，每行一个相对文件名，支持 `#` 注释。 |

---

## 3. 技术栈

| 层级 | 选型 | 说明 |
|------|------|------|
| 编程语言 | **Rust** | 高性能、内存安全、生态丰富。 |
| Web 框架 | **actix-web** | 用户已有使用经验，异步性能优异，中间件生态完善。 |
| 数据库 ORM | **sqlx** | 编译期 SQL 检查，原生异步支持，无需额外代码生成。通过 `DbPool` 枚举运行时切换 SQLite / PostgreSQL / MySQL，无需重新编译。 |
| 数据库驱动 | **sqlx** 内置 | `sqlx-sqlite`、`sqlx-postgres`、`sqlx-mysql` 三个 feature 默认全部启用，运行时通过 `database.driver` 配置项选择。 |
| 序列化 | **serde** | JSON / TOML 解析与生成。 |
| XML 生成 | **quick-xml** + **serde** | 快速、低内存占用，支持流式写入大型 XML。多命名空间与美化输出的写法已经 POC 实测（详见 6.3 节"XML 序列化设计"）。 |
| HTTP 客户端 | **reqwest** | 异步 HTTP 客户端，支持重试、超时。 |
| 日志 | **tracing** + **tracing-subscriber** | Rust 生态标准日志方案，支持结构化输出和级别控制。 |
| 配置 | **toml** + **serde** | 配置文件采用 TOML 格式。 |
| 命令行 | **clap** | 功能强大的命令行解析库，支持子命令、参数校验。 |
| 时间处理 | **chrono** | 日期时间解析与格式化。 |
| 数据库迁移 | **sqlx-cli** / 或内置迁移脚本 | 建议 `sqlx migrate` 管理数据库 schema 版本。 |

---

## 4. 系统架构

```
┌─────────────────────────────────────────────────────────────────┐
│                          cu-scanner                             │
├─────────────────────────────────────────────────────────────────┤
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐           │
│  │   CLI 模块   │  │   API 模块    │  │  同步调度模块 │          │
│  │  (clap)      │  │ (actix-web)  │  │  (tokio cron)│          │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘          │
│         │                 │                 │                  │
│  ┌──────┴─────────────────┴─────────────────┴───────┐          │
│  │              核心服务层 (Service Layer)           │          │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐          │          │
│  │  │ 转换引擎  │ │ 下载服务  │ │ 合并服务  │         │          │
│  │  └──────────┘ └──────────┘ └──────────┘          │          │
│  └──────────────────────────────────────────────────┘          │
│         │                │                │                    │
│  ┌──────┴────────────────┴────────────────┴───────┐            │
│  │              数据访问层 (Repository)            │            │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐        │            │
│  │  │ 公告元数据│ │OVAL 定义 │ │ 下载记录  │        │            │
│  │  └──────────┘ └──────────┘ └──────────┘        │            │
│  └────────────────────────────────────────────────┘            │
│         │                │                │                    │
│  ┌──────┴────────────────┴────────────────┴───────┐            │
│  │              存储层 (Storage)                  │             │
│  │  MySQL / PostgreSQL / SQLite                   │            │
│  └────────────────────────────────────────────────┘            │
└────────────────────────────────────────────────────────────────┘
```

### 4.1 模块职责

| 模块 | 职责 |
|------|------|
| **CLI** | 解析命令行参数，调用转换引擎完成单文件/目录转换，不依赖数据库。 |
| **API** | actix-web 路由处理，参数校验，调用服务层，返回 XML 或 JSON 错误。 |
| **同步调度** | 定时（或手动触发）读取配置中的 `index.txt` URL，解析索引，下载新增/变更 CSAF 文件，调用转换引擎入库。 |
| **转换引擎** | 解析 CSAF JSON，提取产品和漏洞信息，生成 OVAL 的 `definitions` / `tests` / `objects` / `states` XML 结构。 |
| **下载服务** | HTTP 下载，支持重试、断点续传（可选）、并发控制，解析 `index.txt`（过滤 `#` 注释）。 |
| **合并服务** | 按月份或时间段查询多个 OVAL 定义，合并为单个 XML（共享 generator、合并 definitions/tests/objects/states、去重）。 |
| **Repository** | 数据库 CRUD，封装 `sqlx` 查询，实现多数据库兼容。 |

---

## 5. 数据模型

### 5.1 CSAF JSON 输入结构（关键字段）

```json
{
  "document": {
    "title": "openssh security update",
    "csaf_version": "2.0",
    "category": "csaf_vex",
    "publisher": { "name": "ChinaUnicom", "category": "vendor", "namespace": "..." },
    "tracking": {
      "id": "CUOS-SA-2025-1665",
      "version": "1.0.0",
      "status": "final",
      "initial_release_date": "2025-11-10T12:03:14+08:00",
      "current_release_date": "2025-11-10T12:03:14+08:00"
    },
    "aggregate_severity": { "text": "Low", "namespace": "..." },
    "notes": [ { "text": "...", "category": "summary" } ],
    "references": [ { "summary": "CVE-2025-32728", "url": "...", "category": "external" } ]
  },
  "product_tree": {
    "branches": [
      {
        "name": "ChinaUnicom", "category": "vendor",
        "branches": [ { "name": "CUOS", "category": "product_name", "branches": [ ... ] } ]
      }
    ],
    "relationships": [
      {
        "category": "default_component_of",
        "full_product_name": { "name": "...", "product_id": "CUOS-4.0:openssh-..." },
        "product_reference": "openssh-9.6p1-6.ule4.aarch64.rpm",
        "relates_to_product_reference": "CUOS-4.0"
      }
    ]
  },
  "vulnerabilities": [
    {
      "cve": "CVE-2025-32728",
      "notes": [ { "text": "...", "category": "description" } ],
      "product_status": { "fixed": [ "CUOS-4.0:openssh-...", ... ] },
      "remediations": [
        {
          "category": "vendor_fix",
          "details": "Update to the fixed version for CVE-2025-32728",
          "product_ids": [ ... ],
          "url": "https://www.chinaunicom.com/security/advisories/CVE-2025-32728"
        }
      ],
      "scores": [ { "cvss_v3": { "version": "3.1", "vectorString": "...", "baseScore": 3.8, "baseSeverity": "LOW" } } ],
      "threats": [ { "category": "impact", "details": "Low" } ]
    }
  ]
}
```

### 5.2 OVAL XML 输出结构

一个标准的 OVAL 补丁定义文件（如 `cuos-openssh-20251665.oval.xml`）结构如下：

```xml
<?xml version="1.0" encoding="utf-8"?>
<oval_definitions
  xmlns="http://oval.mitre.org/XMLSchema/oval-definitions-5"
  xmlns:oval="http://oval.mitre.org/XMLSchema/oval-common-5"
  xmlns:unix-def="http://oval.mitre.org/XMLSchema/oval-definitions-5#unix"
  xmlns:red-def="http://oval.mitre.org/XMLSchema/oval-definitions-5#linux"
  xmlns:ind-def="http://oval.mitre.org/XMLSchema/oval-definitions-5#independent"
  xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
  xsi:schemaLocation="...">

  <generator>
    <oval:product_name>cu-scanner</oval:product_name>
    <oval:product_version>1.0.0</oval:product_version>
    <oval:schema_version>5.10</oval:schema_version>
    <oval:timestamp>2025-11-10T12:03:14Z</oval:timestamp>
    <oval:content_version>1754331613</oval:content_version>
  </generator>

  <definitions>
    <definition class="patch" id="oval:com.chinaunicom.cuos:def:20251665" version="1">
      <metadata>
        <title>CUOS-SA-2025-1665: openssh security update (Low)</title>
        <affected family="unix">
          <platform>CUOS 4.0</platform>
        </affected>
        <reference ref_id="CUOS-SA-2025-1665" ref_url="..." source="CSAF"/>
        <reference ref_id="CVE-2025-32728" ref_url="..." source="CVE"/>
        <description>...</description>
        <advisory from="security@chinaunicom.com">
          <severity>Low</severity>
          <rights>Copyright 2025 ChinaUnicom, Inc.</rights>
          <issued date="2025-11-10"/>
          <updated date="2025-11-10"/>
          <cve cvss3="3.8/CVSS:3.1/AV:L/AC:L/PR:L/UI:N/S:C/C:N/I:L/A:N" href="...">CVE-2025-32728</cve>
          <affected_cpe_list>
            <cpe>cpe:/o:chinaunicom:cuos:4.0</cpe>
          </affected_cpe_list>
        </advisory>
      </metadata>
      <criteria operator="AND">
        <!-- 平台检测 -->
        <criterion comment="CUOS 4.0 is installed" test_ref="oval:com.chinaunicom.cuos:tst:202516650001"/>
        <criteria operator="OR">
          <criteria operator="AND">
            <criterion comment="openssh is earlier than 0:9.6p1-6.ule4" test_ref="oval:com.chinaunicom.cuos:tst:202516650101"/>
            <criterion comment="openssh is signed with ChinaUnicom key" test_ref="oval:com.chinaunicom.cuos:tst:202516650102"/>
          </criteria>
          <!-- 其他子包检测... -->
        </criteria>
      </criteria>
    </definition>
  </definitions>

  <tests>
    <red-def:rpmverifyfile_test ... id="oval:com.chinaunicom.cuos:tst:202516650001" ...>
      <red-def:object .../>
      <red-def:state .../>
    </red-def:rpmverifyfile_test>
    <red-def:rpminfo_test ... id="oval:com.chinaunicom.cuos:tst:202516650201" ...>
      <red-def:object .../>
      <red-def:state .../>
    </red-def:rpminfo_test>
    <!-- ... -->
  </tests>

  <objects>
    <red-def:rpminfo_object id="oval:com.chinaunicom.cuos:obj:202516650001" ...>
      <red-def:name>openssh</red-def:name>
    </red-def:rpminfo_object>
    <!-- ... -->
  </objects>

  <states>
    <red-def:rpminfo_state id="oval:com.chinaunicom.cuos:ste:202516650001" ...>
      <red-def:evr datatype="evr_string" operation="less than">0:9.6p1-6.ule4</red-def:evr>
    </red-def:rpminfo_state>
    <!-- ... -->
  </states>
</oval_definitions>
```

#### 5.2.1 命名空间与 ID 前缀

- **OVAL Definition ID**: `oval:com.chinaunicom.cuos:def:{numeric_id}`
  - `CUOS-SA-2025-1665` → `20251665`（去掉连字符和 `CUOS-SA-` 前缀）
- **Test ID**: `oval:com.chinaunicom.cuos:tst:{numeric_id}{seq}`
- **Object ID**: `oval:com.chinaunicom.cuos:obj:{numeric_id}{seq}`
- **State ID**: `oval:com.chinaunicom.cuos:ste:{numeric_id}{seq}`
> **注意**：`{seq}` 为 4 位序列号（0001-9999），仅用于 Test / Object / State ID；Definition ID 不带 seq。详见 6.3 节 OVAL ID 生成规则。
- **Generator**:
  - `product_name`: `cu-scanner`
  - `schema_version`: `5.10`
  - `content_version`: 当前时间戳（Unix 秒级）

#### 5.2.2 合并 OVAL 的规范

当合并多个 OVAL 文件（按月份或时间段）时，需生成一个**复合 OVAL 文件**，结构如下：

1. **单个 `<generator>`**：使用当前时间生成新的 generator 信息。
2. **合并 `<definitions>`**：将各文件的 `<definition>` 依次放入，保持原有 ID 和版本。
3. **合并 `<tests>` / `<objects>` / `<states>`**：
   - **去重**：如果多个 definition 引用了相同的 test/object/state（通过 ID 判断），只保留一份。
   - 例如，检测 "CUOS 4.0 is installed" 的 test 可能出现在多个 definition 中，合并后只保留一个。
4. **Schema 声明**：保持与单个文件一致的 `xmlns` 和 `xsi:schemaLocation`。

合并后的文件示例：

```xml
<?xml version="1.0" encoding="utf-8"?>
<oval_definitions ...>
  <generator>
    <oval:product_name>cu-scanner</oval:product_name>
    <oval:product_version>1.0.0</oval:product_version>
    <oval:schema_version>5.10</oval:schema_version>
    <oval:timestamp>2025-12-01T00:00:00Z</oval:timestamp>
    <oval:content_version>1764470400</oval:content_version>
  </generator>
  <definitions>
    <definition id="oval:com.chinaunicom.cuos:def:20251665" ...>...</definition>
    <definition id="oval:com.chinaunicom.cuos:def:20251666" ...>...</definition>
    <!-- ... -->
  </definitions>
  <tests><!-- 去重后的所有 tests --></tests>
  <objects><!-- 去重后的所有 objects --></objects>
  <states><!-- 去重后的所有 states --></states>
</oval_definitions>
```

### 5.3 数据库表结构

采用 **元数据拆分存储** 策略，不存储完整 XML 文本，而是将 OVAL 的五大组件（definitions/tests/objects/states）及其关系拆解为关系型表，在 API 查询时动态组装为 XML 输出。

> **设计理由**：
> - 支持按月/按时间段灵活查询和合并；
> - 支持去重、增量更新、版本管理；
> - 避免存储大量重复 XML 文本。
>
> **写入事务要求**：单个 CSAF 的入库/覆盖更新涉及 `csaf_sources`、`oval_definitions`、`oval_references`、`oval_cves`、`oval_cpes`、`oval_tests`、`oval_objects`、`oval_states`、`oval_criteria` 等多张表，**必须包裹在同一个数据库事务中提交**；任何一步失败整体回滚，禁止出现"定义已写入但 criteria 缺失"的中间态。覆盖更新时先在事务内逻辑作废旧版本行（或标记 `superseded`），再写入新版本行。

#### 表：`csaf_sources`（CSAF 原始文件记录）

| 字段 | 类型 | 说明 |
|------|------|------|
| `id` | `SERIAL / INTEGER PRIMARY KEY` | 自增主键 |
| `csaf_id` | `VARCHAR(64) UNIQUE NOT NULL` | CSAF 公告 ID，如 `CUOS-SA-2025-1665` |
| `file_name` | `VARCHAR(256) NOT NULL` | 原始文件名，如 `csaf-cuos-sa-2025-1665.json` |
| `title` | `TEXT` | 公告标题 |
| `category` | `VARCHAR(32)` | CSAF 类别，如 `csaf_vex` |
| `severity` | `VARCHAR(16)` | 严重级别：Low / Moderate / Important / Critical |
| `release_date` | `TIMESTAMP` | `current_release_date` |
| `csaf_version` | `VARCHAR(8)` | CSAF 版本，如 `2.0` |
| `tracking_version` | `VARCHAR(16)` | 追踪版本，如 `1.0.0` |
| `download_url` | `TEXT` | 下载来源 URL |
| `downloaded_at` | `TIMESTAMP` | 下载时间 |
| `parsed_at` | `TIMESTAMP` | 解析入库时间 |
| `oval_numeric_id` | `VARCHAR(16)` | 对应的 OVAL 数字 ID，如 `20251665` |
| `created_at` | `TIMESTAMP` | 记录创建时间 |
| `updated_at` | `TIMESTAMP` | 记录更新时间 |

#### 表：`oval_definitions`（OVAL 定义元数据）

| 字段 | 类型 | 说明 |
|------|------|------|
| `id` | `SERIAL PRIMARY KEY` | 自增主键 |
| `csaf_id` | `VARCHAR(64) NOT NULL` | 关联 `csaf_sources.csaf_id` |
| `oval_id` | `VARCHAR(128) NOT NULL` | 完整 OVAL ID，如 `oval:com.chinaunicom.cuos:def:20251665`；与 `version` 组成**复合唯一键** `UNIQUE(oval_id, version)`（不能用单列 UNIQUE，否则与"同一 CSAF 保留多版本"的归档策略冲突） |
| `class` | `VARCHAR(16)` | `patch`（固定值） |
| `version` | `INTEGER` | 版本号，初始为 1 |
| `title` | `TEXT` | 定义标题 |
| `description` | `TEXT` | 描述文本 |
| `family` | `VARCHAR(16)` | `unix`（固定值） |
| `platform` | `VARCHAR(128)` | 平台名称，如 `CUOS 4.0` |
| `severity` | `VARCHAR(16)` | 严重级别 |
| `issued_date` | `DATE` | 发布日期 |
| `updated_date` | `DATE` | 更新日期 |
| `rights` | `TEXT` | 版权信息 |
| `advisory_from` | `VARCHAR(128)` | 发件人邮箱 |
| `generator_timestamp` | `TIMESTAMP` | 生成时间戳 |
| `content_version` | `BIGINT` | 内容版本（时间戳） |
| `created_at` | `TIMESTAMP` | 创建时间 |
| `updated_at` | `TIMESTAMP` | 更新时间 |

#### 表：`oval_references`（OVAL Definition 中的引用）

| 字段 | 类型 | 说明 |
|------|------|------|
| `id` | `SERIAL PRIMARY KEY` | 自增主键 |
| `definition_id` | `INTEGER NOT NULL` | 关联 `oval_definitions.id` |
| `ref_id` | `VARCHAR(64)` | 引用 ID，如 `CUOS-SA-2025-1665` / `CVE-2025-32728` |
| `ref_url` | `TEXT` | 引用 URL |
| `source` | `VARCHAR(16)` | `CSAF` / `CVE` / `CWE` |

#### 表：`oval_cves`（OVAL Definition 中的 CVE 详情）

| 字段 | 类型 | 说明 |
|------|------|------|
| `id` | `SERIAL PRIMARY KEY` | 自增主键 |
| `definition_id` | `INTEGER NOT NULL` | 关联 `oval_definitions.id` |
| `cve_id` | `VARCHAR(32)` | CVE 编号，如 `CVE-2025-32728` |
| `cvss3` | `VARCHAR(64)` | CVSS 向量字符串 |
| `impact` | `VARCHAR(16)` | 影响级别 |
| `href` | `TEXT` | CVE 链接 |
| `public_date` | `DATE` | 公开日期 |

#### 表：`oval_cpes`（OVAL Definition 中的 CPE 列表）

| 字段 | 类型 | 说明 |
|------|------|------|
| `id` | `SERIAL PRIMARY KEY` | 自增主键 |
| `definition_id` | `INTEGER NOT NULL` | 关联 `oval_definitions.id` |
| `cpe` | `VARCHAR(256)` | CPE 字符串 |

#### 表：`oval_tests`（OVAL Tests）

| 字段 | 类型 | 说明 |
|------|------|------|
| `id` | `SERIAL PRIMARY KEY` | 自增主键 |
| `oval_id` | `VARCHAR(128) UNIQUE NOT NULL` | 完整 OVAL Test ID，如 `oval:com.chinaunicom.cuos:tst:202516650001` |
| `definition_id` | `INTEGER NOT NULL` | 关联 `oval_definitions.id` |
| `test_type` | `VARCHAR(32)` | `rpminfo_test` / `rpmverifyfile_test` |
| `check` | `VARCHAR(32)` | `at least one` / `none satisfy` |
| `comment` | `TEXT` | 注释说明 |
| `version` | `INTEGER` | 版本号 |
| `object_ref` | `VARCHAR(128)` | 关联 Object ID |
| `state_ref` | `VARCHAR(128)` | 关联 State ID |
| `created_at` | `TIMESTAMP` | 创建时间 |

> **关联关系说明**：`oval_tests` 通过 `definition_id` 与 `oval_definitions` 直接关联；`oval_objects` / `oval_states` 通过 `oval_tests` 间接关联（`test.object_ref` → `object.oval_id`，`test.state_ref` → `state.oval_id`）。在**合并去重场景**下，多个 `definition` 可能引用相同的 `test` / `object` / `state`（如平台检测）。数据库层保留原始关联，合并时通过内存中的 ID 去重逻辑处理，不修改数据库关系。
>
> **组件表唯一性与覆盖更新的配合**：由于 OVAL ID 由 `numeric_id + seq` 派生、同一 CSAF 各版本保持一致，`oval_tests` / `oval_objects` / `oval_states` 的 `oval_id UNIQUE` 约束意味着**组件行在库中全局只有一份**。因此覆盖更新采用"**组件删除重建**"策略：事务内先删除旧版本 definition 关联的全部 tests/objects/states/criteria/references/cves/cpes，再按新版本重新生成插入；`oval_definitions` 主表可按 5.3 节保留多版本历史行，但历史版本的明细组件不保留（查询历史版本时如需 XML，从归档冷存储恢复）。API 查询/合并始终只针对最新版本的组件集合。
> 
> 如果后续需要数据库级别的去重查询，建议新增关联表：
> ```sql
> CREATE TABLE definition_tests (
>     definition_id INTEGER NOT NULL REFERENCES oval_definitions(id),
>     test_id INTEGER NOT NULL REFERENCES oval_tests(id),
>     PRIMARY KEY (definition_id, test_id)
> );
> ```

#### 表：`oval_objects`（OVAL Objects）

| 字段 | 类型 | 说明 |
|------|------|------|
| `id` | `SERIAL PRIMARY KEY` | 自增主键 |
| `oval_id` | `VARCHAR(128) UNIQUE NOT NULL` | 完整 OVAL Object ID |
| `object_type` | `VARCHAR(32)` | `rpminfo_object` / `rpmverifyfile_object` |
| `name` | `VARCHAR(128)` | 包名或文件名 |
| `rpm_version` | `VARCHAR(128)` | RPM 包版本或文件版本（rpmverifyfile 用） |
| `filepath` | `TEXT` | 文件路径（rpmverifyfile 用） |
| `version` | `INTEGER` | OVAL 组件版本号（如 `version="1"`） |
| `created_at` | `TIMESTAMP` | 创建时间 |

#### 表：`oval_states`（OVAL States）

| 字段 | 类型 | 说明 |
|------|------|------|
| `id` | `SERIAL PRIMARY KEY` | 自增主键 |
| `oval_id` | `VARCHAR(128) UNIQUE NOT NULL` | 完整 OVAL State ID |
| `state_type` | `VARCHAR(32)` | `rpminfo_state` / `rpmverifyfile_state` |
| `evr` | `VARCHAR(128)` | EVR 字符串，如 `0:9.6p1-6.ule4` |
| `evr_operation` | `VARCHAR(16)` | `less than` / `equals` / `pattern match` |
| `signature_keyid` | `VARCHAR(64)` | 签名 Key ID |
| `name_pattern` | `VARCHAR(128)` | 名称匹配模式 |
| `version_pattern` | `VARCHAR(128)` | 版本匹配模式 |
| `version` | `INTEGER` | 版本号 |
| `created_at` | `TIMESTAMP` | 创建时间 |

#### 表：`oval_criteria`（OVAL Criteria 树结构）

| 字段 | 类型 | 说明 |
|------|------|------|
| `id` | `SERIAL PRIMARY KEY` | 自增主键 |
| `definition_id` | `INTEGER NOT NULL` | 关联 `oval_definitions.id` |
| `parent_id` | `INTEGER` | 父节点 ID，NULL 表示顶层（根 criteria） |
| `operator` | `VARCHAR(8)` | `AND` / `OR` |
| `criterion_test_ref` | `VARCHAR(128)` | 如果是 criterion 叶子节点，引用 test ID |
| `criterion_comment` | `TEXT` | criterion 注释 |
| `sequence` | `INTEGER` | 同级节点排序序号 |
| `created_at` | `TIMESTAMP` | 创建时间 |

> **说明**：`oval_criteria` 以**邻接表模型**存储 criteria 树，每个节点可以是 `criteria`（非叶子，有 operator）或 `criterion`（叶子，引用 test）。通过 `parent_id` 构建树形结构。

#### 表：`users`（API 用户与凭证）

> 对应第 13 章 JWT 认证。用户由管理员通过 CLI 管理（见 6.1 节 `user` 子命令），系统不提供自助注册接口。

| 字段 | 类型 | 说明 |
|------|------|------|
| `id` | `SERIAL PRIMARY KEY` | 自增主键 |
| `username` | `VARCHAR(64) UNIQUE NOT NULL` | 登录用户名，正则 `^[a-zA-Z0-9_.-]{3,64}$` |
| `password_hash` | `VARCHAR(128) NOT NULL` | **bcrypt 哈希**（cost ≥ 12），严禁明文或可逆加密存储登录口令 |
| `role` | `VARCHAR(16) NOT NULL` | `admin`（可管理用户/触发写操作）/ `user`（只读查询），写入 JWT Claims |
| `status` | `VARCHAR(16)` | `active` / `disabled`（禁用即无法登录，已签发 Token 可配合 13.7 黑名单立即失效） |
| `failed_attempts` | `INTEGER DEFAULT 0` | 连续登录失败次数（配合 9.7 限流，超过阈值临时锁定） |
| `locked_until` | `TIMESTAMP` | 锁定截止时间，NULL 表示未锁定 |
| `last_login_at` | `TIMESTAMP` | 最近登录时间 |
| `must_change_password` | `BOOLEAN DEFAULT false` | 管理员创建/重置口令后置 `true`，用户下次登录后必须经 `POST /auth/password` 改密（见 13.3.3、13.8 节） |
| `created_at` / `updated_at` | `TIMESTAMP` | 创建 / 更新时间 |

> **引导管理员**：首次启动时若 `users` 表为空，从环境变量 `CU_SCANNER_ADMIN_USER` / `CU_SCANNER_ADMIN_PASSWORD` 创建初始 admin（bcrypt 哈希后入库），未设置则拒绝启用认证并输出明显告警——**不允许内置默认口令**。

#### 表：`rpm_epoch_cache`（RPM Epoch 查询缓存）

> 对应 6.3 节 Epoch 缓存机制与 14.1 节缓存策略，原设计引用了该表但未给出结构，此处补齐。

| 字段 | 类型 | 说明 |
|------|------|------|
| `id` | `SERIAL PRIMARY KEY` | 自增主键 |
| `pkg_name` | `VARCHAR(128) NOT NULL` | RPM 包名 |
| `repo_id` | `VARCHAR(128) NOT NULL` | yum 源标识（不同源的 epoch 可能不同），与 `pkg_name` 组成复合唯一键 `UNIQUE(pkg_name, repo_id)` |
| `epoch` | `VARCHAR(16) NOT NULL` | 查询到的 epoch 值（通常为 `"0"`） |
| `source` | `VARCHAR(16)` | 获取途径：`dnf` / `repodata` / `api` / `manual` / `default` |
| `resolved_at` | `TIMESTAMP` | 查询时间（用于缓存失效判断，如超过 30 天重新查询） |
| `created_at` | `TIMESTAMP` | 创建时间 |

#### 表：`download_tasks`（下载同步记录）

| 字段 | 类型 | 说明 |
|------|------|------|
| `id` | `SERIAL PRIMARY KEY` | 自增主键 |
| `file_name` | `VARCHAR(256) NOT NULL` | index.txt 中的文件名 |
| `source_url` | `TEXT` | 完整下载 URL |
| `status` | `VARCHAR(16)` | `pending` / `success` / `failed` / `skipped` |
| `attempt_count` | `INTEGER` | 重试次数 |
| `error_message` | `TEXT` | 失败原因 |
| `sync_batch_id` | `VARCHAR(64)` | 同步批次 ID（时间戳或 UUID） |
| `created_at` | `TIMESTAMP` | 创建时间 |
| `completed_at` | `TIMESTAMP` | 完成时间 |

---

#### 数据库索引设计

为确保查询性能，以下字段必须建立索引：

| 表名 | 索引字段 | 索引类型 | 说明 |
|------|----------|----------|------|
| `csaf_sources` | `csaf_id` | `UNIQUE` | 公告唯一标识查询 |
| `csaf_sources` | `oval_numeric_id` | `INDEX` | OVAL 数字 ID 反向查询 |
| `csaf_sources` | `release_date` | `INDEX` | 按日期范围查询 |
| `oval_definitions` | `(oval_id, version)` | `UNIQUE` | 复合唯一键：允许同一 OVAL ID 保留多版本（增量同步覆盖更新时插入新版本行） |
| `oval_definitions` | `oval_id` | `INDEX` | OVAL ID 精确查询（取最新版本：`WHERE oval_id = ? ORDER BY version DESC LIMIT 1`） |
| `oval_definitions` | `csaf_id` | `INDEX` | 关联 CSAF 查询 |
| `oval_definitions` | `issued_date` | `INDEX` | 按月份/时间段合并查询（最频繁） |
| `oval_references` | `definition_id` | `INDEX` | 关联 Definition 查询 |
| `oval_cves` | `definition_id` | `INDEX` | 关联 Definition 查询 |
| `oval_cves` | `cve_id` | `INDEX` | CVE 编号查询 |
| `oval_cpes` | `definition_id` | `INDEX` | 关联 Definition 查询 |
| `oval_tests` | `oval_id` | `UNIQUE` | Test ID 精确查询 |
| `oval_tests` | `definition_id` | `INDEX` | 关联 Definition 查询 |
| `oval_objects` | `oval_id` | `UNIQUE` | Object ID 精确查询 |
| `oval_states` | `oval_id` | `UNIQUE` | State ID 精确查询 |
| `oval_criteria` | `definition_id` | `INDEX` | 按 Definition 查询 criteria 树 |
| `oval_criteria` | `parent_id` | `INDEX` | 递归构建树结构 |
| `download_tasks` | `file_name` | `INDEX` | 文件名查询 |
| `download_tasks` | `sync_batch_id` | `INDEX` | 按批次查询 |
| `download_tasks` | `status` | `INDEX` | 按状态筛选 |
| `download_tasks` | `created_at` | `INDEX` | 自动清理（删除超过保留期的历史记录） |
| `rpm_epoch_cache` | `(pkg_name, repo_id)` | `UNIQUE` | 缓存查询主键 |
| `users` | `username` | `UNIQUE` | 登录查询 |

> **性能注意**：`issued_date` 索引对合并查询至关重要（`GET /oval/month/{year-month}` 和 `GET /oval/range`）。如使用 MySQL/PostgreSQL，建议对 `issued_date` 建立**复合索引** `(issued_date, updated_at)` 以支持"取最新版本"的查询。

#### 数据归档与清理策略

OVAL 定义数据会持续增长，需制定归档策略：

| 策略 | 说明 |
|------|------|
| **自动清理** | 超过 2 年的 `download_tasks` 记录自动删除（可通过配置调整） |
| **结果归档** | 超过 1 年的 OVAL 定义可导出为静态 XML 文件归档，从数据库中移除明细但保留索引 |
| **版本控制** | 同一 CSAF 的多个版本保留最新 3 个历史版本，更早版本归档到冷存储 |

---

## 6. 核心模块设计

### 6.1 命令行模块（CLI）

使用 `clap` 定义子命令：

```bash
cu-scanner convert --input <path> --output <path>
cu-scanner server --config <path>
cu-scanner sync --config <path>
cu-scanner user <add|list|passwd|disable|enable>   # API 用户管理（见 13.8 节）
```

#### 子命令：`convert`

| 参数 | 短参 | 长参 | 类型 | 必填 | 默认值 | 说明 |
|------|------|------|------|------|--------|------|
| 输入 | `-i` | `--input` | `Path` | 是 | - | 输入文件或目录路径 |
| 输出 | `-o` | `--output` | `Path` | 是 | - | 输出文件或目录路径 |
| 覆盖 | `-f` | `--force` | `bool` | 否 | `false` | 强制覆盖已存在的输出文件 |
| 详细 | `-v` | `--verbose` | `bool` | 否 | `false` | 输出 DEBUG 级别日志 |

**行为逻辑**：
1. 如果 `--input` 是文件：直接转换单文件，输出到 `--output`（如果 `--output` 是目录，按规则命名后写入）。
2. 如果 `--input` 是目录：遍历目录下所有 `.json` 文件，逐一转换到 `--output` 目录。
3. 输出目录不存在时自动创建。
4. 输出文件已存在时：默认跳过并打印提示；若 `--force` 则覆盖。
5. **命名规则**：`csaf-cuos-sa-2025-1665.json` → `cuos-openssh-20251665.oval.xml`
   - 前缀 `cuos-`（小写）
   - 中间取该 CSAF 文件中第一个软件包名称（去掉版本和架构，如 `openssh`）
   - 后缀为 OVAL 数字 ID（如 `20251665`）
   - 扩展名 `.oval.xml`
   - **注意**：如果第一个软件包名本身包含 `-`（如 `openssh-askpass`），完整保留，不截断。例如 `cuos-openssh-askpass-20251665.oval.xml`。

#### 子命令：`server`

| 参数 | 短参 | 长参 | 类型 | 必填 | 默认值 | 说明 |
|------|------|------|------|------|--------|------|
| 配置 | `-c` | `--config` | `Path` | 是 | - | TOML 配置文件路径 |

**行为逻辑**：
1. 读取 TOML 配置文件，初始化数据库连接池、日志、HTTP 服务。
2. 启动 actix-web 服务，监听配置中指定的地址和端口。
3. 可选：启动时自动执行一次 `sync`（如果配置中 `sync.on_startup = true`）。
4. 可选：按配置中的定时表达式（如 `0 0 */6 * * *`，tokio-cron-scheduler 6 字段格式，首字段为秒）自动触发后台同步任务。

#### 子命令：`sync`

| 参数 | 短参 | 长参 | 类型 | 必填 | 默认值 | 说明 |
|------|------|------|------|------|--------|------|
| 配置 | `-c` | `--config` | `Path` | 是 | - | TOML 配置文件路径 |

**行为逻辑**：
1. `sync` 子命令执行**一次性同步**后退出，不启动定时任务（定时同步通过 `server` 子命令 + 配置 `sync.cron` 实现）。
2. 从配置读取 `sync.index_url`（父目录地址，如 `https://www.chinaunicom.com/security/advisories/`）。
3. 拼接 `index.txt` 完整 URL（`index_url` + `index.txt`），下载并解析有效文件名（忽略 `#` 注释和空行）。
4. 对每个文件名：
   - 提取 CSAF ID（如 `csaf-cuos-sa-2025-1665.json` → `CUOS-SA-2025-1665`）。
   - 查询 `csaf_sources` 表：
     - 如果不存在：转下载-入库流程。
     - 如果已存在：先**仅下载远程文件到内存/临时区**，解析其 `tracking.version` 与本地比较：
       - 远程版本更新：覆盖本地记录（按 5.3 节事务要求整体更新数据库和 OVAL 定义，新增版本行）。
       - 版本相同：跳过，丢弃临时文件，不产生写操作。
   - 下载-入库流程：拼接完整 CSAF 下载 URL（`index_url` + 文件名），下载 JSON 文件。
   - 下载失败时按指数退避重试，最多 `sync.max_retries` 次，初始间隔 `sync.retry_interval_sec` 秒。
   - 下载成功：解析 CSAF → 转换为 OVAL → 写入/更新数据库（单事务提交）。

### 6.2 下载同步模块

```rust
pub struct DownloadService {
    client: reqwest::Client,
    config: SyncConfig,
    repository: DownloadTaskRepository,
}

impl DownloadService {
    /// 下载 index.txt 并解析文件名列表
    /// index_url 为父目录，自动拼接 index.txt
    pub async fn fetch_index(&self, index_url: &str) -> Result<Vec<String>, Error>;

    /// 下载单个 CSAF 文件
    /// base_url 为父目录，自动拼接文件名
    pub async fn download_file(&self, base_url: &str, file_name: &str) -> Result<Bytes, Error>;

    /// 执行完整同步流程
    pub async fn sync(&self) -> Result<SyncReport, Error>;
}
```

**并发控制**：同步下载时建议限制并发数（如 `sync.concurrent_limit = 10`），防止对远程服务器造成压力。

**重试策略**：指数退避（与 14.2 节一致）：初始间隔 `sync.retry_interval_sec`（默认 2s），之后按 2 的幂递增（2s → 4s → 8s → 16s → 32s），最多重试 `sync.max_retries` 次（默认 5）。对 4xx 类客户端错误（如 404）不重试，直接标记 failed。

### 6.3 转换引擎模块

```rust
pub struct Converter;

impl Converter {
    /// 将 CSAF JSON 字符串解析为内部结构
    pub fn parse_csaf(json: &str) -> Result<CsafDocument, Error>;

    /// 将 CSAF 转换为 OVAL 结构（内存模型）
    pub fn convert(csaf: &CsafDocument) -> Result<OvalDocument, Error>;

    /// 将 OVAL 内存模型序列化为 XML 字符串（pretty = true 时输出缩进格式）
    pub fn serialize(oval: &OvalDocument, pretty: bool) -> Result<String, Error>;

    /// 将 OVAL 内存模型流式写入任意 writer（用于合并大文件 / HTTP 流式响应，
    /// 避免全量 XML 驻留内存，见 14.1 节）
    pub fn serialize_to<W: std::io::Write>(oval: &OvalDocument, writer: W, pretty: bool) -> Result<(), Error>;
}
```

#### XML 序列化设计（quick-xml + serde，已通过 POC 实测验证）

> 以下结论基于 `quick-xml 0.38` + `serde 1.0` 的最小验证程序（`xml-ns-poc/`）实测确认，编码时可直接按此模式实现。

**1. 多命名空间支持（字面透传模式）**

quick-xml 不做命名空间解析/校验，把标签与属性名作为字符串原样写出，因此 OVAL 的多命名空间声明按"普通属性"处理：

| 需求 | 实现手段 |
|------|----------|
| 默认命名空间 `xmlns="..."` | 根结构体字段 `#[serde(rename = "@xmlns")]` |
| 前缀声明 `xmlns:red-def="..."` | 字段 `#[serde(rename = "@xmlns:red-def")]` |
| 带前缀元素 `<red-def:rpminfo_test>` | `#[serde(rename = "red-def:rpminfo_test")]` |
| `xsi:schemaLocation` | `#[serde(rename = "@xsi:schemaLocation")]` |
| `<tests>`/`<objects>`/`<states>` 内混合多种组件类型 | **外部标签枚举**（enum 每个 variant `rename` 为不同标签名）+ 容器字段 `#[serde(rename = "$value")]` |

```rust
#[derive(Serialize)]
#[serde(rename = "oval_definitions")]
struct OvalDefinitions {
    #[serde(rename = "@xmlns")]
    xmlns: String,               // "http://oval.mitre.org/XMLSchema/oval-definitions-5"
    #[serde(rename = "@xmlns:red-def")]
    xmlns_red_def: String,       // "http://oval.mitre.org/XMLSchema/oval-definitions-5#linux"
    // ... 其余 xmlns 声明同模式
    tests: Tests,
}

#[derive(Serialize)]
struct Tests {
    #[serde(rename = "$value")]
    items: Vec<Test>,            // 异构元素列表
}

#[derive(Serialize)]
enum Test {
    #[serde(rename = "red-def:rpminfo_test")]
    RpmInfo { /* ... */ },
    #[serde(rename = "red-def:rpmverifyfile_test")]
    RpmVerifyFile { /* ... */ },
}
```

**注意事项**：
- 前缀为字面透传，**拼写错误不会报错**（如误写 `red_def:`），必须靠单元测试比对样例文件 + OVAL schema 校验（`xmllint`）把关；
- 属性输出顺序 = 结构体字段声明顺序，xmlns 声明字段放在根结构体最前；
- quick-xml **不支持** serde 的内部/相邻标签枚举（`#[serde(tag = "...")]`），异构列表只能用外部标签枚举；
- 属性字段统一用 `@` 前缀，文本内容用 `$text`，混合内容列表用 `$value`（不同版本关键字有差异，以所锁定版本的文档为准，并在 Cargo.toml 中锁定 quick-xml 主版本）。

**2. 美化输出（缩进）**

serde 序列化使用 `Serializer::indent()`（quick-xml ≥ 0.31）：

```rust
use quick_xml::se::Serializer;

let mut buf = String::new();
let mut ser = Serializer::new(&mut buf);
ser.indent(' ', 2);                 // 2 空格缩进；不调用则输出紧凑格式
doc.serialize(ser)?;                // 注意传值，不是 &mut ser
```

已实测的约束：
- `se::Serializer` 的 writer 要求实现 `std::fmt::Write`（即写入 `String`），**不能**直接套 `quick_xml::Writer`——不要组合 `Serializer::new(Writer::new_with_indent(...))`；
- 文本节点保持单行内联（如 `<red-def:evr ...>0:9.6p1-6.ule4</red-def:evr>` 不会被拆行），格式干净；
- CLI 输出文件默认开启缩进（便于人工审阅与 diff）；**API 响应建议关闭缩进**（减小传输体积）。

**3. 流式输出与内存控制（配合 14.1 节）**

| 场景 | 方式 | 说明 |
|------|------|------|
| 单文件转换、小规模合并（≤ 100 个 definition） | serde → `String`（方式 A） | 实现简单，全量内存可接受 |
| 大规模合并（最多 5000 个 definition）、HTTP 流式响应 | `quick_xml::Writer::new_with_indent(writer, b' ', 2)` 手写事件流（方式 B），按 definition 分块写出 | 避免全量 XML 驻留内存；xmlns 声明手工写一次，各组件逐条写事件 |

方式 B 中命名空间同样以字面属性写入根元素起始标签（`BytesStart::new("oval_definitions").push_attribute(("xmlns:red-def", "..."))`），与方式 A 输出格式保持一致。

#### RPM Epoch 获取方案（关键设计）

CSAF JSON 中仅包含 RPM 文件名（如 `openssh-9.6p1-6.ule4.aarch64.rpm`），从中可以解析出 Name、Version、Release、Arch，但 **无法获取 Epoch 值**。OVAL 的 EVR 字符串完整格式为 `epoch:version-release`，缺少 Epoch 会导致检测不准确。

**解决方案：通过 yum/dnf 源查询 Epoch**

```rust
pub struct YumEpochResolver {
    cache: HashMap<String, String>, // pkg_name -> epoch
    repo_config: Option<String>,     // repo 文件路径或 URL
}

impl YumEpochResolver {
    /// 初始化 resolver，加载缓存
    pub fn new(repo_config: Option<String>) -> Self;
    
    /// 查询指定包的 epoch
    /// 优先查缓存，未命中则调 dnf/yum 查询
    pub async fn resolve_epoch(&mut self, pkg_name: &str) -> Result<String, Error>;
    
    /// 批量预查询（在转换前统一查一批包的 epoch，减少开销）
    pub async fn preload_epochs(&mut self, pkg_names: &[&str]) -> Result<(), Error>;
}
```

**实现策略**：

| 策略 | 命令/方式 | 说明 | 优先级 |
|------|----------|------|--------|
| **本地 dnf 查询** | `dnf repoquery --qf '%{EPOCH}' {pkg_name}` | 如果服务器运行在有 yum/dnf 的 CUOS/openEuler 环境 | 1 |
| **解析 repodata** | 下载 `repodata/primary.xml.gz` 或 SQLite db，解析 `<epoch>` 标签 | 如果服务器无 dnf 环境，但可访问 yum 源 | 2 |
| **远程 API 查询** | 调用内部包管理 API（如果联通有） | 需要自定义实现 | 3 |
| **默认值** | `epoch = "0"` | 上述均失败时的兜底 | 4 |

> **模块设计**：此策略在一个**单独文件或独立 crate 中实现**（如 `epoch-resolver` 模块），对外暴露调用接口，供转换引擎和 CLI 调用。

**缓存机制**：
- 内存缓存：转换进程内 HashMap，避免同一批次重复查询。
- 数据库缓存：在 `oval_objects` 表或新增 `rpm_epoch_cache` 表中持久化，`epoch` 值通常不会变更，可长期复用。

**配置示例**（`config.toml`）：
```toml
[yum]
# 是否启用 epoch 查询（禁用则默认使用 0）
enabled = true
# 本地 yum/dnf 的 repo 配置文件路径（可选，不填则使用系统默认）
repo_config = "/etc/yum.repos.d/cuos.repo"
# 远程 yum 源基础 URL（用于 repodata 解析，作为 dnf 查询的备选）
base_url = "https://repo.cucloud.com/cuos/4.0/os/"
# 查询超时（秒）
timeout_sec = 30
# 是否将查询结果缓存到数据库
cache_to_db = true
```

> **注意**：命令行模式（CLI）同样支持 epoch 查询，但如果在无 dnf 环境的机器上运行，建议通过 `--epoch-map` 参数传入手动指定的 epoch 映射，或启用远程 repodata 解析。

#### 平台检测设计（支持 CUOS / openEuler）

OVAL 的 `criteria` 需要包含平台检测逻辑，确保只在目标系统上执行漏洞检测。

**检测方式**：使用 `rpmverifyfile_test` 检测 `/etc/os-release` 文件内容。

**CUOS 检测**：

```xml
<red-def:rpmverifyfile_test check="at least one" comment="CUOS is installed"
  id="oval:com.chinaunicom.cuos:tst:{def_id}0001" version="1">
  <red-def:object object_ref="oval:com.chinaunicom.cuos:obj:{def_id}0001"/>
  <red-def:state state_ref="oval:com.chinaunicom.cuos:ste:{def_id}0001"/>
</red-def:rpmverifyfile_test>

<red-def:rpmverifyfile_object id="oval:com.chinaunicom.cuos:obj:{def_id}0001" version="1">
  <red-def:behaviors .../>
  <red-def:name operation="pattern match">^os-release$</red-def:name>
  <red-def:filepath>/etc/os-release</red-def:filepath>
</red-def:rpmverifyfile_object>

<red-def:rpmverifyfile_state id="oval:com.chinaunicom.cuos:ste:{def_id}0001" version="1">
  <red-def:name operation="pattern match">^os-release$</red-def:name>
  <red-def:version operation="pattern match">^CUOS</red-def:version>
</red-def:rpmverifyfile_state>
```

> **说明**：红帽示例中使用 `rpmverifyfile_test` 检测 `/etc/redhat-release`。对于 CUOS，检测 `/etc/os-release` 更标准。`rpmverifyfile_state` 的 `version` 元素用于匹配文件内容中的版本信息（如 `NAME="CUOS"`）。

**openEuler 检测**（当 CSAF 中平台为 openEuler 时）：

```xml
<red-def:rpmverifyfile_state ...>
  <red-def:name operation="pattern match">^os-release$</red-def:name>
  <red-def:version operation="pattern match">^openEuler|^.*ID="openEuler"</red-def:version>
</red-def:rpmverifyfile_state>
```

**平台检测在 criteria 中的结构**：

```xml
<criteria operator="AND">
  <!-- 平台检测：CUOS 4.0 已安装 -->
  <criterion comment="CUOS 4.0 is installed" test_ref="..."/>
  <criteria operator="OR">
    <!-- 各漏洞包检测（含签名验证） -->
    <criteria operator="AND">
      <criterion comment="openssh is earlier than {epoch}:{evr}" test_ref="..."/>
      <criterion comment="openssh is signed with CUOS key" test_ref="..."/>
    </criteria>
    ...
  </criteria>
</criteria>
```

> **平台版本检测**：必须精确到版本（如仅 CUOS 4.0），在 `rpmverifyfile_state` 中增加 `VERSION_ID="4.0"` 的匹配，或在 `rpminfo_test` 中检测 `cuos-release` 包的版本。因为后续大版本升级会升级到 CUOS 5.0，不同版本的漏洞定义不能混淆。

#### 签名验证设计

**规则**：
- **服务模式**：如果 `config.toml` 中 `[oval]` 段配置了 `signature_keyid`，则在生成 OVAL 时自动添加签名检测 criterion；未配置则不添加。
- **命令行模式**：不添加签名检测 criterion（即忽略 `signature_keyid` 配置）。

**OVAL 签名检测结构**：

```xml
<red-def:rpminfo_test check="at least one" comment="{pkg} is signed with CUOS key"
  id="oval:com.chinaunicom.cuos:tst:{def_id}{seq}" version="1">
  <red-def:object object_ref="oval:com.chinaunicom.cuos:obj:{def_id}{seq}"/>
  <red-def:state state_ref="oval:com.chinaunicom.cuos:ste:{def_id}{seq}"/>
</red-def:rpminfo_test>

<red-def:rpminfo_state id="oval:com.chinaunicom.cuos:ste:{def_id}{seq}" version="1">
  <red-def:signature_keyid operation="equals">{signature_keyid}</red-def:signature_keyid>
</red-def:rpminfo_state>
```

#### CSAF → OVAL 字段映射

| CSAF 字段 | OVAL 对应位置 | 转换规则 |
|-----------|--------------|----------|
| `document.tracking.id` | `definition.id` | `CUOS-SA-YYYY-NNNN` → `oval:com.chinaunicom.cuos:def:YYYYNNNN` |
| `document.title` | `definition.metadata.title` | 前缀拼接，如 `CUOS-SA-2025-1665: {title} ({severity})` |
| `document.publisher.name` | `definition.metadata.advisory.from` | `security@chinaunicom.com`（固定） |
| `document.tracking.current_release_date` | `definition.metadata.advisory.issued/updated` | 取日期部分 `YYYY-MM-DD` |
| `document.aggregate_severity.text` | `definition.metadata.advisory.severity` | 直接映射 |
| `document.notes` | `definition.metadata.description` | `category=description` 或 `summary` 的 note 文本 |
| `document.references` | `definition.metadata.reference` | `category=self` → `source=CSAF`；`category=external` + CVE URL → `source=CVE` |
| `product_tree` | `definition.metadata.affected` / `affected_cpe_list` | 提取 `vendor` / `product_name` / `product_version` 构建 CPE 和 platform |
| `vulnerabilities[].cve` | `definition.metadata.advisory.cve` | 提取 CVE 编号、CVSS 向量、impact |
| `vulnerabilities[].product_status.fixed` | `criteria` / `tests` / `objects` / `states` | 生成 RPM 版本检测逻辑（含 epoch 查询） |
| `vulnerabilities[].remediations[].details` | `definition.metadata.description` 附加 | 追加修复说明 |

#### RPM 包检测逻辑生成（关键设计）

CSAF 的 `product_tree` 包含产品（RPM 包）信息，如 `openssh-9.6p1-6.ule4.aarch64.rpm`。转换引擎需从中解析：

1. **包名（Name）**：`openssh`、`openssh-askpass`、`openssh-clients` 等。
2. **版本（Version）**：`9.6p1`
3. **发布号（Release）**：`6.ule4`
4. **架构（Arch）**：`aarch64`、`x86_64`、`loongarch64`、`noarch`
5. **EVR 字符串**：`{epoch}:{version}-{release}`（epoch 通过 yum 源查询获取）

**解析规则**（RPM 文件名格式：`name-version-release.arch.rpm`）：
- 由于 `name` 本身可能包含 `-`（如 `openssh-askpass`），解析需从右向左：
  - 最后一段去掉 `.rpm` 是 `arch`。
  - 倒数第二段是 `release`（通常包含 `-` 和数字）。
  - 倒数第三段是 `version`（通常包含数字和字母）。
  - 剩余部分是 `name`。
- 示例：`openssh-askpass-9.6p1-6.ule4.aarch64.rpm`
  - arch: `aarch64`
  - release: `6.ule4`
  - version: `9.6p1`
  - name: `openssh-askpass`

**OVAL 检测逻辑生成**（每个 fixed 包名生成一组）：

```xml
<!-- Test: 检测包版本是否小于修复版本 -->
<red-def:rpminfo_test check="at least one" comment="{name} is earlier than {epoch}:{evr}"
  id="oval:com.chinaunicom.cuos:tst:{def_id}{seq}" version="1">
  <red-def:object object_ref="oval:com.chinaunicom.cuos:obj:{def_id}{seq}"/>
  <red-def:state state_ref="oval:com.chinaunicom.cuos:ste:{def_id}{seq}"/>
</red-def:rpminfo_test>

<!-- Object: 指定包名 -->
<red-def:rpminfo_object id="oval:com.chinaunicom.cuos:obj:{def_id}{seq}" version="1">
  <red-def:name>{name}</red-def:name>
</red-def:rpminfo_object>

<!-- State: 版本小于修复版本 -->
<red-def:rpminfo_state id="oval:com.chinaunicom.cuos:ste:{def_id}{seq}" version="1">
  <red-def:evr datatype="evr_string" operation="less than">{epoch}:{version}-{release}</red-def:evr>
</red-def:rpminfo_state>
```

**Criteria 构建**：

```xml
<criteria operator="AND">
  <!-- 平台检测：CUOS 已安装 -->
  <criterion comment="CUOS {version} is installed" test_ref="..."/>
  <criteria operator="OR">
    <!-- 各漏洞包检测（含签名验证，服务模式下可选） -->
    <criteria operator="AND">
      <criterion comment="{pkg} is earlier than {epoch}:{evr}" test_ref="..."/>
      <criterion comment="{pkg} is signed with CUOS key" test_ref="..."/>
    </criteria>
    ...
  </criteria>
</criteria>
```

#### OVAL ID 生成规则

#### 1 基础格式

对于每个 CSAF 文件 `CUOS-SA-YYYY-NNNN`，提取 **OVAL 数字 ID**：`YYYYNNNN`（如 `20251665`）。

**ID 格式**：

```
oval:com.chinaunicom.cuos:{type}:{numeric_id}{seq}
```

| 占位符 | 说明 | 示例 |
|--------|------|------|
| `{type}` | 组件类型 | `def` / `tst` / `obj` / `ste` |
| `{numeric_id}` | 8 位数字（从 CSAF ID 提取） | `20251665` |
 | `{seq}` | 4 位序列号（0001-9999） | `0001` |

**示例**：
- Definition: `oval:com.chinaunicom.cuos:def:20251665`
- Test: `oval:com.chinaunicom.cuos:tst:202516650001`
- Object: `oval:com.chinaunicom.cuos:obj:202516650001`
- State: `oval:com.chinaunicom.cuos:ste:202516650001`

#### 2 序列号 {seq} 分配规则（4位，已更新）

| 序号范围 | 用途 | 数量 | 说明 | 对应 Test 类型 |
|----------|------|------|------|----------------|
| **0001-0100** | 平台检测 | 100个 | 支持多个发行版/版本，如 CUOS 4.0, CUOS 5.0, openEuler 20.03, 22.03, 24.03 等 | `rpmverifyfile_test` |
| **0101-0200** | 签名检测 | 100个 | 仅服务模式且配置 `signature_keyid` 时生成 | `rpminfo_test` |
| **0201-9999** | 版本检测 | 9799个 | 每个 `fixed` 包生成一个 | `rpminfo_test` |
| **10000+** | 预留扩展 | - | 未来扩展，需升级为5位 | - |

> **设计说明**：
> - 同一 CSAF 可能同时影响多个发行版/版本（如 CUOS 4.0 和 openEuler 22.03），每个平台版本需要独立的平台检测 test，因此保留 100 个平台检测编号。
> - 平台检测按 `(发行版, 版本)` 对分配唯一序号，序号映射规则：
>   - CUOS 4.0 → 0001, CUOS 5.0 → 0002, ...
>   - openEuler 20.03 → 0020, openEuler 22.03 → 0022, openEuler 24.03 → 0024, ...
> - 签名检测和版本检测各自保留 100 / 9799 个，正常场景足够使用。

#### 3 完整示例

**CSAF**: `CUOS-SA-2025-1665`
**OVAL 数字 ID**: `20251665`
**包列表**: openssh, openssh-clients, openssh-server（3 个包）

##### Definition ID

```
oval:com.chinaunicom.cuos:def:20251665
```

##### 平台检测（seq = 0001）

检测 `/etc/os-release` 确认 CUOS/openEuler 是否安装：

```xml
<red-def:rpmverifyfile_test check="at least one" comment="CUOS 4.0 is installed"
  id="oval:com.chinaunicom.cuos:tst:202516650001" version="1">
  <red-def:object object_ref="oval:com.chinaunicom.cuos:obj:202516650001"/>
  <red-def:state state_ref="oval:com.chinaunicom.cuos:ste:202516650001"/>
</red-def:rpmverifyfile_test>

<red-def:rpmverifyfile_object id="oval:com.chinaunicom.cuos:obj:202516650001" version="1">
  <red-def:behaviors noconfigfiles="true" .../>
  <red-def:name operation="pattern match">^os-release$</red-def:name>
  <red-def:filepath>/etc/os-release</red-def:filepath>
</red-def:rpmverifyfile_object>

<red-def:rpmverifyfile_state id="oval:com.chinaunicom.cuos:ste:202516650001" version="1">
  <red-def:name operation="pattern match">^os-release$</red-def:name>
  <red-def:version operation="pattern match">^CUOS|^openEuler</red-def:version>
</red-def:rpmverifyfile_state>
```

> **说明**：合并时多个 definition 的平台检测可能相同（如都是 CUOS 4.0），按 OVAL ID 去重后只保留一份。

##### 签名检测（seq = 0101-0103，假设启用）

每个包一个签名检测（共 3 个）：

| 包名 | Test ID | Object ID | State ID |
|------|---------|-----------|----------|
| openssh | `tst:202516650101` | `obj:202516650101` | `ste:202516650101` |
| openssh-clients | `tst:202516650102` | `obj:202516650102` | `ste:202516650102` |
| openssh-server | `tst:202516650103` | `obj:202516650103` | `ste:202516650103` |

```xml
<!-- 示例：openssh 签名检测 -->
<red-def:rpminfo_test check="at least one" comment="openssh is signed with CUOS key"
  id="oval:com.chinaunicom.cuos:tst:202516650101" version="1">
  <red-def:object object_ref="oval:com.chinaunicom.cuos:obj:202516650101"/>
  <red-def:state state_ref="oval:com.chinaunicom.cuos:ste:202516650101"/>
</red-def:rpminfo_test>

<red-def:rpminfo_object id="oval:com.chinaunicom.cuos:obj:202516650101" version="1">
  <red-def:name>openssh</red-def:name>
</red-def:rpminfo_object>

<red-def:rpminfo_state id="oval:com.chinaunicom.cuos:ste:202516650101" version="1">
  <red-def:signature_keyid operation="equals">{signature_keyid}</red-def:signature_keyid>
</red-def:rpminfo_state>
```

> **注意**：签名检测仅在服务模式且配置 `signature_keyid` 时生成；CLI 模式不生成。

##### 版本检测（seq = 0201-0203）

每个包一个版本检测（共 3 个）：

| 包名 | Test ID | Object ID | State ID |
|------|---------|-----------|----------|
| openssh | `tst:202516650201` | `obj:202516650201` | `ste:202516650201` |
| openssh-clients | `tst:202516650202` | `obj:202516650202` | `ste:202516650202` |
| openssh-server | `tst:202516650203` | `obj:202516650203` | `ste:202516650203` |

```xml
<!-- 示例：openssh 版本检测 -->
<red-def:rpminfo_test check="at least one" comment="openssh is earlier than 0:9.6p1-6.ule4"
  id="oval:com.chinaunicom.cuos:tst:202516650201" version="1">
  <red-def:object object_ref="oval:com.chinaunicom.cuos:obj:202516650201"/>
  <red-def:state state_ref="oval:com.chinaunicom.cuos:ste:202516650201"/>
</red-def:rpminfo_test>

<red-def:rpminfo_object id="oval:com.chinaunicom.cuos:obj:202516650201" version="1">
  <red-def:name>openssh</red-def:name>
</red-def:rpminfo_object>

<red-def:rpminfo_state id="oval:com.chinaunicom.cuos:ste:202516650201" version="1">
  <red-def:evr datatype="evr_string" operation="less than">0:9.6p1-6.ule4</red-def:evr>
</red-def:rpminfo_state>
```

#### 4 关联关系

```
+--------------+       +---------------+       +---------------+
|  definition  |       |     test      |       |    object     |
|  def:2025... |       | tst:202516... |       | obj:202516... |
+--------------+       +---------------+       +---------------+
      |                        |                       |
      | criteria               | object_ref            |
      | (criterion)            | state_ref             |
      v                        v                       v
+--------------+       +---------------+       +---------------+
|  criteria    |       |     state     |       |   (包名)      |
|  test_ref    |       | ste:202516... |       |  openssh      |
+--------------+       +---------------+       +---------------+
```

- `test.object_ref` → `object.oval_id`
- `test.state_ref` → `state.oval_id`
- `criteria.criterion_test_ref` → `test.oval_id`

#### 5 去重策略

合并多个 OVAL 文件时：

| 去重维度 | 规则 | 示例 |
|----------|------|------|
| 按 OVAL ID | 相同 `oval_id` 的 test/object/state 只保留一份 | 多个 definition 的平台检测都是 `tst:202516650001`，合并后只保留一个 |
| 按内容 | 如果 ID 不同但内容相同（如相同的包名+版本），也保留（因为 ID 不同，被引用的 definition 不同） | 不同 CSAF 对 openssh 的版本检测 ID 不同，各自保留 |

#### 6 Rust 实现设计

```rust
pub struct IdGenerator {
    numeric_id: String,
    next_signature_seq: u32, // 从 0101 开始
    next_version_seq: u32,   // 从 0201 开始
}

impl IdGenerator {
    pub fn new(numeric_id: &str) -> Self {
        Self {
            numeric_id: numeric_id.to_string(),
            next_signature_seq: 101,
            next_version_seq: 201,
        }
    }
    
    /// 生成 Definition ID
    pub fn def_id(&self) -> String {
        format!("oval:com.chinaunicom.cuos:def:{}", self.numeric_id)
    }
    
    /// 平台检测 ID（固定 0001，按平台版本映射）
    pub fn platform_ids(&self, platform_seq: u32) -> (String, String, String) {
        self._ids(platform_seq)
    }
    
    /// 签名检测 ID（seq 从 0101 开始递增）
    pub fn next_signature_ids(&mut self) -> (String, String, String) {
        let seq = self.next_signature_seq;
        self.next_signature_seq += 1;
        self._ids(seq)
    }
    
    /// 版本检测 ID（seq 从 0201 开始递增）
    pub fn next_version_ids(&mut self) -> (String, String, String) {
        let seq = self.next_version_seq;
        self.next_version_seq += 1;
        self._ids(seq)
    }
    
    fn _ids(&self, seq: u32) -> (String, String, String) {
        let seq_str = format!("{:04}", seq);
        (
            format!("oval:com.chinaunicom.cuos:tst:{}{}", self.numeric_id, seq_str),
            format!("oval:com.chinaunicom.cuos:obj:{}{}", self.numeric_id, seq_str),
            format!("oval:com.chinaunicom.cuos:ste:{}{}", self.numeric_id, seq_str),
        )
    }
}

// 使用示例
let mut gen = IdGenerator::new("20251665");
let def_id = gen.def_id();
let (plat_tst, plat_obj, plat_ste) = gen.platform_ids(1);   // seq=0001, CUOS 4.0
let (sig_tst, sig_obj, sig_ste) = gen.next_signature_ids(); // seq=0101
let (ver_tst, ver_obj, ver_ste) = gen.next_version_ids();   // seq=0201
```

#### 7 数据库中存储的关联

| 表 | 字段 | 示例值 |
|----|------|--------|
| `oval_definitions` | `oval_id` | `oval:com.chinaunicom.cuos:def:20251665` |
| `oval_tests` | `oval_id` | `oval:com.chinaunicom.cuos:tst:202516650001` |
| `oval_tests` | `object_ref` | `oval:com.chinaunicom.cuos:obj:202516650001` |
| `oval_tests` | `state_ref` | `oval:com.chinaunicom.cuos:ste:202516650001` |
| `oval_objects` | `oval_id` | `oval:com.chinaunicom.cuos:obj:202516650001` |
| `oval_states` | `oval_id` | `oval:com.chinaunicom.cuos:ste:202516650001` |
| `oval_criteria` | `criterion_test_ref` | `oval:com.chinaunicom.cuos:tst:202516650001` |

#### 8 注意事项

1. **序列号上限**：单个 CSAF 最多支持 100 个平台检测 + 100 个签名检测 + 9799 个版本检测，正常场景足够使用。
2. **超出范围**：如果包数量超过范围，使用 `9999` 并记录警告，或考虑扩展为 5 位序列号。
3. **合并时保持 ID 不变**：合并后的 OVAL XML 中，每个 definition 引用的 test/object/state ID 保持原样，通过 ID 去重。
4. **ID 唯一性**：`numeric_id` + `seq` 的组合保证全局唯一。
5. **数字 ID 位数边界**：`numeric_id` 取自 `CUOS-SA-YYYY-NNNN`，当公告序号 `NNNN` 超过 9999（同一年公告数破万）时数字 ID 将变为 9 位以上，与固定 4 位 `seq` 拼接后无法从总长度反推边界。要求：`numeric_id` 与 `seq` 必须**按固定宽度字段处理**（seq 恒为 4 位零填充，从右往左截取），数据库另行存储拆分后的 `oval_numeric_id` 列用于查询，禁止靠字符串长度解析。

### 6.4 数据库模块（Repository）

使用 `sqlx` 实现异步数据库访问，通过 `#[sqlx::migrate]` 或 `sqlx-cli` 管理迁移。

### DbPool — 运行时三数据库切换

`DbPool` 是一个三变体枚举，支持通过配置文件在运行时选择数据库，**无需重新编译**：

```rust
#[derive(Clone)]
pub enum DbPool {
    Sqlite(SqlitePool),
    Postgres(PgPool),
    Mysql(MySqlPool),
}

impl DbPool {
    /// 根据 config.driver 创建对应连接池
    /// driver: "sqlite" | "postgres" | "mysql"
    pub async fn create(config: &DatabaseConfig) -> Result<Self>;
    /// 开始事务
    pub async fn begin(&self) -> Result<DbTransaction<'_>>;
}

pub enum DbTransaction<'a> {
    Sqlite(sqlx::Transaction<'a, sqlx::Sqlite>),
    Postgres(sqlx::Transaction<'a, sqlx::Postgres>),
    Mysql(sqlx::Transaction<'a, sqlx::MySql>),
}
```

### 查询宏

由于 `DbPool` 是枚举而非单一类型，所有 sqlx 查询必须通过宏包装以匹配具体变体。7 个查询宏定义在 `src/db/pool.rs`：

| 宏 | 返回类型 | 用途 |
|-----|---------|------|
| `pool_exec!(pool, sql, params...)` | `Result<u64>` | 写入（返回影响行数） |
| `pool_insert_id!(pool, sql, params...)` | `Result<i64>` | INSERT 返回新行 ID |
| `pool_fetch!(pool, Type, sql, params...)` | `Result<Vec<Type>>` | 查询多行 |
| `pool_fetch_opt!(pool, Type, sql, params...)` | `Result<Option<Type>>` | 查询可选行 |
| `pool_fetch_one!(pool, Type, sql, params...)` | `Result<Type>` | 查询单行 |
| `tx_exec!(tx, sql, params...)` | `Result<u64>` | 事务内写入 |
| `tx_insert_id!(tx, sql, params...)` | `Result<i64>` | 事务内 INSERT 返回 ID |

**跨数据库差异由宏内部处理**：
- SQLite: `?` 占位符，`last_insert_rowid()` 获取 ID
- PostgreSQL: `?` 占位符，`RETURNING id` 子句自动追加获取 ID
- MySQL: `?` 占位符，`last_insert_id()` 获取 ID

> **SQLite 并发说明**：SQLite 为单写者模型，服务模式中同步任务写库与 API 读库会互相阻塞。启用 SQLite 时**必须在连接串或初始化时开启 WAL 模式**（`sqlite://./cu-scanner.db?mode=rwc` + `PRAGMA journal_mode=WAL; PRAGMA busy_timeout=5000;`），以支持读写并发并避免 `database is locked` 错误。节点规模或并发量较大时建议直接使用 MySQL/PostgreSQL。
>
> **时间字段规范**：所有 `TIMESTAMP` 字段统一存 UTC（`chrono::Utc`），展示层再做时区转换；SQLite 无时区类型，禁止混存本地时间。
>
> **Cargo.toml features**：三个数据库驱动默认全部编译：
> ```toml
> [features]
> default = ["sqlite", "postgres", "mysql"]
> sqlite = ["sqlx/sqlite"]
> postgres = ["sqlx/postgres"]
> mysql = ["sqlx/mysql"]
> ```

### Repository 模板

```rust
use crate::db::{DbPool, DbTransaction};
use crate::{pool_fetch, pool_fetch_opt, pool_insert_id, tx_exec, tx_insert_id};

pub struct CsafSourceRepository { pool: DbPool }

impl CsafSourceRepository {
    pub fn new(pool: DbPool) -> Self { Self { pool } }
    
    pub async fn find_by_csaf_id(&self, csaf_id: &str) -> Result<Option<CsafSourceRow>> {
        pool_fetch_opt!(&self.pool, CsafSourceRow, "SELECT * FROM csaf_sources WHERE csaf_id = ?", csaf_id)
    }
    
    pub async fn insert(&self, tx: &mut DbTransaction<'_>, record: &InsertCsafSource) -> Result<i64> {
        tx_insert_id!(tx, "INSERT INTO csaf_sources (...) VALUES (?,?,...)", &record.field1, &record.field2)
    }
}
```

### 6.5 HTTP API 模块（actix-web）

```rust
pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg
        .service(web::resource("/health").get(health_handler))
        .service(web::resource("/oval/{id}").get(get_oval_by_id))
        .service(web::resource("/oval/month/{year-month}").get(get_oval_by_month))
        .service(web::resource("/oval/range").get(get_oval_by_range))
        .service(web::resource("/csaf").post(upload_csaf_handler))  // 上传 CSAF 入库（7.2.6）
        // JWT 认证接口（见第 13 章；中间件对 /oval/* 与 /csaf 强制校验，/health 豁免）
        .service(web::resource("/auth/login").post(login_handler))
        .service(web::resource("/auth/refresh").post(refresh_handler))
        .service(web::resource("/auth/password").post(change_password_handler));  // 自助改密（13.3.3）
}
```

---

## 7. API 接口设计

### 7.1 通用响应格式

| 场景 | Content-Type | 响应体 |
|------|-------------|--------|
| 成功返回 OVAL XML | `application/xml` | 标准 OVAL XML 文档 |
| 错误（参数校验、时间跨度过大、数据不存在） | `application/json` | `{"error": "...", "code": "..."}` |
| 未认证 / Token 无效或过期 | `application/json` | `{"error": "...", "code": "UNAUTHORIZED"}`（HTTP 401） |

> **设计说明**：用户提到可能使用 `wget` 直接下载 XML，因此成功时必须返回纯净的 XML，不要包裹 JSON。错误时返回 JSON 便于程序解析。

> **认证全局约定（详见第 13 章）**：
> - 除 `/health`、`/metrics` 外，**所有接口（含 `/oval/*`、`POST /csaf`）均要求 JWT 认证**：请求头携带 `Authorization: Bearer <token>`；
> - Token 通过 `POST /auth/login` 获取（见 7.2.5），过期后用 `POST /auth/refresh` 刷新；
> - 未携带 Token、Token 过期、签名无效或命中黑名单时，统一返回 `401 Unauthorized`（JSON 错误体），不区分具体原因，避免泄露认证细节；
> - `[auth] enabled = false`（仅本地开发）时上述约束整体关闭。

### 7.2 接口详情

#### 7.2.1 健康检查

```
GET /health
```

> **认证豁免**：`/health`（以及后续的 `/metrics`）不强制 JWT 认证，以便负载均衡/监控系统探活；但应仅绑定内部网络或通过反向代理限制来源 IP，避免暴露内部状态细节。

**响应**：
```json
{
  "status": "ok",
  "version": "1.0.0",
  "database": "connected",
  "timestamp": "2025-12-01T10:00:00Z"
}
```

#### 7.2.2 按 ID 获取单个 OVAL 文件

```
GET /oval/{id}
```

| 参数 | 位置 | 类型 | 必填 | 示例 | 说明 |
|------|------|------|------|------|------|
| `id` | Path | `string` | 是 | `20251665` | OVAL 数字 ID |

**成功响应** (`200 OK`, `Content-Type: application/xml`):
```xml
<?xml version="1.0" encoding="utf-8"?>
<oval_definitions>...</oval_definitions>
```

**错误响应** (`404 Not Found`, `Content-Type: application/json`):
```json
{
  "error": "OVAL definition not found",
  "code": "NOT_FOUND",
  "id": "20251665"
}
```

#### 7.2.3 按月份获取合并 OVAL 文件

```
GET /oval/month/{year-month}
```

| 参数 | 位置 | 类型 | 必填 | 示例 | 说明 |
|------|------|------|------|------|------|
| `year-month` | Path | `string` | 是 | `2025-11` | 年月格式 `YYYY-MM` |

**查询逻辑**：
1. 查询 `oval_definitions` 表中 `issued_date` 落在 `2025-11-01` 到 `2025-11-30` 的所有记录。
2. 如果同一 `csaf_id` 在当月有多个版本，取 `version` 最大的记录（而非 `updated_at`，避免时钟回拨/并发写入导致误判）。
3. 获取这些 definition 关联的所有 tests、objects、states、criteria、references、cves、cpes。
4. 合并为一个 OVAL XML 文件，tests/objects/states 去重。

**成功响应** (`200 OK`, `Content-Type: application/xml`):
```xml
<?xml version="1.0" encoding="utf-8"?>
<oval_definitions>
  <generator>...</generator>
  <definitions>
    <definition id="oval:com.chinaunicom.cuos:def:20251665">...</definition>
    <definition id="oval:com.chinaunicom.cuos:def:20251666">...</definition>
  </definitions>
  <tests><!-- 去重后的所有 tests --></tests>
  <objects><!-- 去重后的所有 objects --></objects>
  <states><!-- 去重后的所有 states --></states>
</oval_definitions>
```

**错误响应** (`404 Not Found`):
```json
{
  "error": "No OVAL definitions found for month 2025-11",
  "code": "NO_DATA",
  "month": "2025-11"
}
```

#### 7.2.4 按时间段获取合并 OVAL 文件

```
GET /oval/range?start=YYYY-MM-DD&end=YYYY-MM-DD
```

| 参数 | 位置 | 类型 | 必填 | 示例 | 说明 |
|------|------|------|------|------|------|
| `start` | Query | `string` | 是 | `2025-06-01` | 开始日期 |
| `end` | Query | `string` | 是 | `2025-11-30` | 结束日期 |

**校验规则**（已确认）：
- 日期格式必须为 `YYYY-MM-DD`。
- 开始日期必须小于等于结束日期。
- **时间跨度不得超过 1 年（365 天）**，否则返回 `400 Bad Request`。

**错误响应** (`400 Bad Request`):
```json
{
  "error": "Date range exceeds maximum allowed span of 1 year",
  "code": "RANGE_TOO_LARGE",
  "start": "2025-06-01",
  "end": "2026-06-01",
  "max_days": 365
}
```

**成功响应** (`200 OK`, `Content-Type: application/xml`):
与月份接口相同，返回合并后的 OVAL XML。

#### 7.2.5 认证接口（JWT）

> 本节只给出接口契约，认证机制完整设计（流程、配置、Rust 实现、Secret 轮换与 Token 撤销）见**第 13 章 API 认证方案**。

**登录获取 Token**：

```
POST /auth/login
Content-Type: application/json

{ "username": "admin", "password": "xxx" }
```

**成功响应** (`200 OK`)：
```json
{
  "token": "eyJhbGciOiJIUzI1NiIs...",
  "token_type": "Bearer",
  "expires_in": 86400
}
```

**刷新 Token**：

```
POST /auth/refresh
Authorization: Bearer <token>
```

成功响应与登录相同，返回新 Token。

**自助修改密码**：

```
POST /auth/password
Authorization: Bearer <token>

{ "old_password": "...", "new_password": "..." }
```

需验证原密码（防 Token 被盗后直接改密），成功后该用户其他历史 Token 加入黑名单；完整契约见 13.3.3 节。

**错误响应** (`401 Unauthorized`)：
```json
{
  "error": "Invalid credentials",
  "code": "UNAUTHORIZED"
}
```

> **速率限制**：`/auth/login` 限流 5 次/分钟/IP（见 9.7、14.1 节），防暴力破解；登录成功/失败均记录审计日志（见 9.10 节）。
>
> **密码传输安全**：
> - 生产环境**必须经 HTTPS 调用本接口**（见 9.8 节），密码只允许出现在 TLS 加密的请求体内，禁止放在 URL/Query 参数中（会被代理与访问日志记录）；
> - **不做客户端哈希**：客户端传明文密码（经 TLS 保护），服务端用 bcrypt 与 `users.password_hash` 比对。客户端预哈希会使哈希值本身沦为"等价口令"，失去意义；
> - 登录失败响应统一为 `Invalid credentials`，不区分"用户不存在"与"密码错误"，防用户名枚举；
> - 密码修改、用户增删通过 CLI `user` 子命令完成（见 6.1 节、13.8 节），不提供自助注册/改密 API。

#### 7.2.6 上传单个 CSAF 文件入库

```
POST /csaf
Authorization: Bearer <token>
```

将单个 CSAF JSON 文件直接通过 HTTP 上传到服务端，完成**校验 → 转换 → 入库**全流程，适用于无法通过远程 `index.txt` 同步、需要人工/第三方系统推送公告的场景。

**请求方式（两种，二选一）**：

| Content-Type | 说明 | 示例 |
|---|---|---|
| `application/json` | 请求体直接为 CSAF JSON 内容（推荐程序化调用） | `curl -H "Authorization: Bearer $T" -H "Content-Type: application/json" --data-binary @csaf-cuos-sa-2025-1665.json https://host/csaf` |
| `multipart/form-data` | 表单字段 `file` 携带文件（兼容传统文件上传组件） | `curl -H "Authorization: Bearer $T" -F "file=@csaf-cuos-sa-2025-1665.json" https://host/csaf` |

**Query 参数**：

| 参数 | 类型 | 必填 | 默认值 | 说明 |
|------|------|------|--------|------|
| `force` | bool | 否 | `false` | `tracking.version` 与库内相同时也强制重建（覆盖更新） |
| `dry_run` | bool | 否 | `false` | 只解析校验并返回转换摘要，**不执行入库**，用于上传前预检 |

**处理逻辑**（与 6.1 节 `sync` 的"下载-入库流程"复用同一 `IngestService`，仅输入来源不同——HTTP 请求体 vs 远程下载）：

1. 校验请求体大小（≤ 10MB，同 9.7 节；超过返回 `413`）。
2. 解析 JSON：语法错误返回 `400 INVALID_JSON`；结构不符合 CSAF 2.0 规范（缺 `document.tracking.id`、`product_tree` 等关键字段）返回 `422 INVALID_CSAF`，错误体中列出缺失/非法字段。
3. 提取 `tracking.id`（CSAF ID）与 `tracking.version`，查询 `csaf_sources`：
   - 不存在 → 新增入库；
   - 已存在且上传版本更新 → 按 5.3 节事务要求覆盖更新（组件删除重建）；
   - 版本相同且 `force=false` → 跳过，返回 `action=skipped`；
   - `dry_run=true` → 执行到转换完成为止，不写库。
4. 转换为 OVAL（含 epoch 查询，超时/失败降级 `epoch=0` 并记入 `warnings`）。
5. **单事务**写入全部关联表（同 sync 入库）。
6. 入库成功后发出"CSAF 更新"事件，供二期自动扫描防抖合并（见二期 6.5 节）。

**成功响应** (`201 Created` 新增 / `200 OK` 更新、跳过或 dry_run，`Content-Type: application/json`)：

```json
{
  "csaf_id": "CUOS-SA-2025-1665",
  "oval_id": "oval:com.chinaunicom.cuos:def:20251665",
  "tracking_version": "1.0.0",
  "action": "created",
  "severity": "Low",
  "release_date": "2025-11-10",
  "summary": {
    "packages": 3,
    "cves": 1,
    "tests": 7,
    "objects": 7,
    "states": 7
  },
  "warnings": [
    "epoch resolved via default fallback (0) for package: openssh-askpass"
  ],
  "duration_ms": 1240
}
```

> `action` 取值：`created`（新增入库，返回 201）/ `updated`（覆盖更新）/ `skipped`（版本相同未变更）/ `dry_run`（仅预检未入库）。

**错误响应**：

| HTTP | code | 场景 |
|------|------|------|
| `400` | `INVALID_JSON` | 请求体不是合法 JSON，或 multipart 缺 `file` 字段 |
| `401` | `UNAUTHORIZED` | 未认证 / Token 无效 |
| `409` | `INGEST_CONFLICT` | 同一 `csaf_id` 正在被并发入库（sync 或另一个上传），稍后重试 |
| `413` | `PAYLOAD_TOO_LARGE` | 请求体超过 10MB |
| `422` | `INVALID_CSAF` | CSAF 结构校验失败，`details.fields` 列出问题字段 |
| `500` | `INTERNAL_ERROR` | 数据库写入失败等内部错误（事务已回滚，库内无残留） |

```json
{
  "error": "CSAF validation failed",
  "code": "INVALID_CSAF",
  "details": {
    "fields": ["document.tracking.id: missing", "product_tree: missing"]
  }
}
```

**设计要点**：
- **幂等性**：重复上传同一文件（同 `csaf_id` + 同 `tracking.version`）返回 `skipped`，不产生重复数据；并发上传由 `(csaf_id)` 唯一约束与事务兜底，冲突方收到 `409` 可安全重试。
- **同步执行**：单文件转换通常在秒级完成，不设异步任务；主要耗时为 epoch 批量预查询（`preload_epochs`），受 `yum.timeout_sec` 上限约束，超时自动降级不阻塞请求。
- **安全**：请求体只在内存解析，**不落盘**（不经过临时文件，无路径遍历面）；全部上传操作记录审计日志（操作人、csaf_id、action、IP，见 9.10 节）；接口需 JWT 认证，建议按 14.1 节限流（默认 30 次/分钟/IP）。
- **与 CLI 的关系**：`convert` 子命令输出文件、不入库；本接口只入库、不返回 OVAL XML。如需上传后立即取回 XML，用 `GET /oval/{id}` 查询。

---

## 8. 配置设计（TOML）

配置文件示例 `config.toml`：

```toml
[server]
# 默认仅绑定回环地址；对外提供服务时由反向代理（Nginx，启用 HTTPS）转发，
# 如需直接监听内网地址请显式修改并配合防火墙策略（见 9.8 节）
host = "127.0.0.1"
port = 8080
workers = 4

[database]
# 运行时选择数据库驱动，无需重新编译
# 支持: "sqlite" | "postgres" (或 "pg") | "mysql"
driver = "sqlite"
# SQLite 示例
url = "sqlite://./cu-scanner.db"
# MySQL 示例（取消注释并修改连接信息即可使用）
# url = "mysql://user:pass@localhost/cu_scanner"
# PostgreSQL 示例
# url = "postgres://user:pass@localhost/cu_scanner"
max_connections = 10
min_connections = 2

[logging]
level = "INFO"  # DEBUG / INFO / WARN / ERROR
format = "text" # text / json（默认 text，可扩展）
file = "./logs/cu-scanner.log"
# 是否输出到控制台
console = true

[sync]
# index.txt 所在父目录的 URL（末尾需带 /）
index_url = "https://www.chinaunicom.com/security/advisories/"
# 是否随服务启动时立即执行一次同步
on_startup = false
# 定时同步的 cron 表达式（可选，为空则不定时同步）
# 注意：tokio-cron-scheduler 使用 6-7 字段格式（秒 分 时 日 月 周 [年]），
# 与传统 crontab 的 5 字段不同，下方示例首字段为"秒"
# cron = "0 0 */6 * * *"
# 最大重试次数
max_retries = 5
# 重试初始间隔（秒），之后指数退避：2s → 4s → 8s → 16s → 32s
retry_interval_sec = 2
# 并发下载数
concurrent_limit = 10
# 下载超时（秒）
timeout_sec = 30

[yum]
# 是否启用 epoch 查询（禁用则默认使用 0）
enabled = true
# 本地 yum/dnf 的 repo 配置文件路径（可选，不填则使用系统默认）
repo_config = "/etc/yum.repos.d/cuos.repo"
# 远程 yum 源基础 URL（用于 repodata 解析，作为 dnf 查询的备选）
base_url = "https://repo.cucloud.com/cuos/4.0/os/"
# 查询超时（秒）
timeout_sec = 30
# 是否将查询结果缓存到数据库
cache_to_db = true

[oval]
# OVAL 命名空间前缀
namespace = "com.chinaunicom.cuos"
# schema 版本
schema_version = "5.10"
# 产品名称（generator）
product_name = "cu-scanner"
# 发件人邮箱
advisory_from = "security@chinaunicom.com"
# 版权信息
rights = "Copyright 2025 ChinaUnicom, Inc."
# 是否生成签名验证 criterion（仅服务模式生效，CLI 忽略）
enable_signature_check = false
# 签名 Key ID（16 位十六进制，如 "199e2f91fd431d51"）
# signature_keyid = ""

[auth]
# API 认证配置
# ⚠️ 默认关闭仅用于本地开发调试；生产环境必须设为 true，否则所有 /oval/* 接口将无认证暴露
enabled = false
# 认证方式：jwt / api_key / oauth2 / mtls
# method = "jwt"
# jwt_secret = "your-secret-key"
# token_expire_hours = 24
```

---

## 9. 安全设计

### 9.1 SQL 注入防护

**风险**：数据库查询拼接用户输入时，可能被注入恶意 SQL。

**防护措施**：
- **使用 `sqlx` 参数化查询**：所有数据库查询均使用 `sqlx` 的编译期检查参数化查询（`query!` / `query_as!`），**禁止字符串拼接 SQL**。
- **查询示例**：
  ```rust
  // 正确：参数化查询
  let row = sqlx::query!("SELECT * FROM oval_definitions WHERE oval_id = ?", oval_id)
      .fetch_one(&pool)
      .await?;
  
  // 错误：字符串拼接（禁止）
  let sql = format!("SELECT * FROM oval_definitions WHERE oval_id = '{}'", oval_id);
  ```
- **输入校验**：所有进入数据库查询的参数必须先经过严格校验（API 路径参数为 OVAL 数字 ID，正则 `^\d{8,}$`，与 9.7 节一致；完整 OVAL ID 由服务端按命名空间前缀在内部组装，不接受客户端直接传入），不合法输入直接拒绝，不进入数据库层。

### 9.2 XSS 防护

**风险**：API 返回的内容可能被浏览器当作 HTML 执行，导致 XSS 攻击。

**防护措施**：
- **正确设置 Content-Type**：
  - 返回 OVAL XML 时：`Content-Type: application/xml`
  - 返回 JSON 错误时：`Content-Type: application/json`
  - 禁止返回 `text/html` 或缺失 Content-Type
- **JSON 序列化**：使用 `serde_json` 自动转义特殊字符（`<`、`>`、`&` 等），避免 JSON 响应中的 XSS。
- **无 HTML 模板渲染**：本系统不返回 HTML 页面，因此不存在服务端模板 XSS 风险。

### 9.3 CSRF 防护

**风险**：浏览器自动携带 Cookie 发起的跨站请求伪造。

**防护措施**：
- **Bearer Token 方案**：API 使用 `Authorization: Bearer <token>` Header 认证，浏览器不会自动携带此 Header，天然免疫 CSRF。
- **无 Cookie 会话**：不使用 Session Cookie，避免 Cookie 相关的 CSRF 风险。
- **CORS 配置**：如需浏览器跨域访问，在 `actix-web` 中严格配置 CORS 白名单（只允许特定域名），禁止 `*` 通配符。
  ```rust
  // actix-cors 配置示例
  Cors::default()
      .allowed_origin("https://internal.cucloud.com")
      .allowed_methods(vec!["GET", "POST"])
      .allowed_headers(vec!["Authorization", "Content-Type"])
  ```

### 9.4 路径遍历防护

**风险**：用户输入路径参数（如 `--output`、文件下载）时，可能构造 `../../../etc/passwd` 等路径逃逸。

**防护措施**：
- **路径规范化**：使用 `std::path::Path::canonicalize()` 或 `std::fs::canonicalize()` 解析路径，确保在预期目录内。
- **路径前缀校验**：CLI 的 `--output` 和 `--input` 参数必须限制在指定目录内，禁止写入系统关键目录。
  ```rust
  pub fn validate_path(input: &str, base_dir: &Path) -> Result<PathBuf, Error> {
      let path = Path::new(input).canonicalize()?;
      if !path.starts_with(base_dir) {
          return Err(Error::PathTraversal);
      }
      Ok(path)
  }
  ```
- **文件扩展名白名单**：CLI 输出只接受 `.oval.xml` 扩展名，不接受其他扩展名。

### 9.5 配置安全

**风险**：配置文件泄露敏感信息（数据库密码、JWT Secret、SSH 私钥路径）。

**防护措施**：
- **配置文件权限**：`config.toml` 建议设置文件权限为 `600`（仅所有者读写），系统在启动时检查权限并警告。
- **Secret 管理**：
  - `jwt_secret` 必须 ≥ 32 字节，建议通过环境变量注入（如 `JWT_SECRET`），而非硬编码在配置文件中。
  - 数据库密码支持通过环境变量（`DATABASE_URL`）传入，避免明文写在配置文件中。
  - **（二期预留）节点 SSH 凭据**：`ssh_password` 属于二期节点扫描功能。此类凭据在使用时必须还原明文，**必须使用可逆加密（AES-256-GCM）存储，严禁使用 bcrypt 等单向哈希**（bcrypt 无法解密还原，仅适用于登录口令校验场景）。加密密钥通过环境变量注入，节点凭据的加密存储与密钥管理细节详见二期规划文档第 5 章数据模型（`nodes` 表 `ssh_password` 字段说明）。
- **环境变量优先**：配置项支持从环境变量覆盖 TOML 中的值（如 `CU_SCANNER_JWT_SECRET`）。

### 9.6 日志安全

**风险**：日志中记录敏感信息（密码、Token、私钥内容、SSH 密码）。

**防护措施**：
- **敏感信息脱敏**：日志中禁止输出以下信息：
  - `jwt_secret`、`database_url` 中的密码部分、Token 值
  - （二期）`ssh_password`、节点密码、SSH 私钥内容（只记录路径，不记录内容）
- **日志访问控制**：日志文件目录（`/var/log/cu-scanner/` 或 `./logs/`）设置权限 `750`，禁止非所有者读取。
- **审计日志**：对认证操作（登录成功/失败）、节点增删改、扫描任务触发记录审计日志，单独存储。

### 9.7 输入校验

**风险**：非法输入导致系统异常、DoS 或安全漏洞。

**防护措施**：
- **严格参数校验**：
  - OVAL ID：正则 `^\d{8,}$`（纯数字，至少8位）
  - 年月格式：正则 `^\d{4}-\d{2}$`
  - 日期格式：`YYYY-MM-DD`，通过 `chrono::NaiveDate::parse_from_str` 校验
  - 文件路径：禁止包含 `\0`（空字节）、控制字符、以及 `..` 序列
- **长度限制**：
  - 文件路径 ≤ 4096 字节
  - 请求体 ≤ 10MB
  - URL 参数长度 ≤ 1024 字节
- **速率限制**：对 `/auth/login` 接口实施速率限制（如 5 次/分钟），防止暴力破解。

### 9.8 网络安全

**风险**：中间人攻击、数据窃听、服务暴露。

**防护措施**：
- **HTTPS 强制**：生产环境必须启用 HTTPS（`TLS 1.2+`），禁止明文 HTTP 暴露 JWT Token。
- **HSTS 头**：如通过反向代理（Nginx）暴露服务，启用 `Strict-Transport-Security`。
- **内部端口绑定**：服务默认绑定 `127.0.0.1` 或内部网络地址，不直接暴露在公网。
- **安全响应头**：通过 `actix-web` 中间件添加：
  ```rust
  app.wrap(middleware::DefaultHeaders::new()
      .add((header::X_CONTENT_TYPE_OPTIONS, "nosniff"))
      .add((header::X_FRAME_OPTIONS, "DENY"))
      .add((header::CONTENT_SECURITY_POLICY, "default-src 'none'")))
  ```

### 9.9 JWT Secret 轮换与 Token 撤销

> 本节内容属于认证机制设计，已归并至 **13.7 节 "Secret 轮换与 Token 撤销"**，此处不再重复。安全设计层面的相关要求（Secret 长度、环境变量注入、HTTPS 强制）见 9.5、9.8 节。

### 9.10 API 访问审计日志

**风险**：无法追溯 API 调用来源，发生安全事件后无法溯源。

**防护措施**：
- **审计日志内容**：每次 API 调用记录以下字段（单独文件或数据表）：
  - 时间戳、请求 ID（`X-Request-ID`）、客户端 IP、User-Agent
  - 认证用户（`sub`）、请求方法、请求路径、请求体摘要（非敏感字段）
  - 响应状态码、响应耗时、错误码（如有）
- **敏感接口强制审计**：以下接口必须记录完整审计日志：
  - `/auth/login`（成功/失败均需记录，密码脱敏）
  - `/auth/refresh`
  - `/auth/password`（改密成功/失败，密码脱敏）
  - `POST /csaf`（记录操作人、`csaf_id`、`action`、来源 IP）
  - 所有 `/oval/*` 查询接口（记录查询参数，用于追溯数据泄露）
- **审计日志保护**：
  - 审计日志文件权限 `600`，单独存储于 `/var/log/cu-scanner/audit/`。
  - 审计日志禁止修改，定期（每日）归档并计算校验和（SHA-256）。

### 9.11 数据库连接安全

**风险**：数据库连接被窃听、中间人攻击导致数据泄露。

**防护措施**：
- **TLS/SSL 连接**：生产环境 MySQL/PostgreSQL 必须启用 TLS 连接，配置 `ssl_mode = require` 或更高。三种数据库驱动默认全部编译进二进制，通过 `database.driver` 配置项运行时选择。
  ```toml
  [database]
  driver = "postgres"   # 运行时选择: "sqlite" | "postgres" | "mysql"
  # PostgreSQL 示例
  url = "postgres://user:pass@host:5432/cu_scanner?sslmode=require"
  # MySQL 示例
  url = "mysql://user:pass@host:3306/cu_scanner?ssl-mode=REQUIRED"
  # SQLite 示例
  url = "sqlite://./cu-scanner.db"
  ```
- **连接加密验证**：配置服务端证书校验（`ssl_ca`），防止伪造数据库服务器。
- **SQLite 安全**：SQLite 文件权限设置为 `640`，禁止组外用户读取。

### 9.12 同步下载安全

**风险**：下载 CSAF 文件时遭遇中间人攻击、DNS 劫持、恶意文件替换。

**防护措施**：
- **HTTPS 强制**：`index_url` 必须为 HTTPS 协议，禁止明文 HTTP。
- **证书校验**：`reqwest` 客户端默认启用系统证书链校验，禁止自定义 `danger_accept_invalid_certs`。
- **下载完整性校验**（可选增强）：
  - 如果 CSAF 源提供 SHA-256 校验文件（如 `index.txt.sha256`），下载后校验文件哈希。
  - 对关键 CSAF 文件，可配置 GPG 签名验证（`signature_url`）。
- **响应大小限制**：限制单次下载响应体大小（如 ≤ 50MB），防止内存耗尽。

### 9.13 数据备份与灾难恢复

**风险**：数据库损坏、误删、磁盘故障导致 OVAL 元数据丢失。

**防护措施**：
- **定期备份**：
  - SQLite：每日冷备份数据库文件（`cp cu-scanner.db cu-scanner.db.bak.YYYYMMDD`），保留 30 天。
  - MySQL/PostgreSQL：依赖数据库原生备份工具（`mysqldump` / `pg_dump`），每日全量备份 + binlog/WAL 增量备份。
- **备份加密**：备份文件使用 AES-256-GCM 加密存储，密钥通过环境变量注入。
- **恢复演练**：每季度执行一次恢复演练，验证备份可用性和 RTO（恢复时间目标 ≤ 2 小时）。

---

## 10. 日志与监控

### 10.1 日志规范

- **使用 `tracing` 框架**：日志级别支持 `TRACE` / `DEBUG` / `INFO` / `WARN` / `ERROR`。
- **格式**：普通文本格式（非结构化 JSON），每条日志包含时间戳、级别、目标模块、消息。
- **示例**：
  ```
  2025-12-01 10:00:00 INFO  cu_scanner::api  Listening on 0.0.0.0:8080
  2025-12-01 10:00:05 DEBUG cu_scanner::sync  Downloading index.txt from https://...
  2025-12-01 10:00:06 WARN  cu_scanner::sync  File csaf-cuos-sa-2025-1665.json already exists, skipping
  ```
- **输出方式**：
  - 控制台输出（`console = true`）
  - 文件输出（`file = "..."`），支持日志轮转（按日期或大小）。
- **级别控制**：通过配置 `logging.level` 控制全局最低输出级别。

### 10.2 监控指标（预留）

建议后续通过 `/metrics` 或 `/health` 暴露以下指标：
- 数据库连接池状态（活跃/空闲连接数）
- 同步任务统计（成功/失败/跳过数）
- API 请求 QPS、延迟、错误率
- 最近一次同步时间

---

## 11. 错误处理

### 11.1 错误分类

| 错误类型 | 说明 | 处理策略 |
|----------|------|----------|
| `ValidationError` | 输入参数校验失败（日期格式、路径不存在等） | 返回 `400 Bad Request`，JSON 错误体 |
| `CsafValidationError` | 上传的 CSAF 结构校验失败（缺关键字段） | 返回 `422 Unprocessable Entity`，错误体列出问题字段 |
| `PayloadTooLargeError` | 上传请求体超过 10MB | 返回 `413 Payload Too Large`，JSON 错误体 |
| `ConflictError` | 同一 `csaf_id` 并发入库冲突（上传与 sync 竞争） | 返回 `409 Conflict`，客户端稍后重试 |
| `NotFoundError` | 请求的资源不存在（OVAL ID、月份无数据） | 返回 `404 Not Found`，JSON 错误体 |
| `RangeError` | 时间范围超过限制（>1年、跨年） | 返回 `400 Bad Request`，JSON 错误体 |
| `DownloadError` | 下载失败（网络超时、HTTP 4xx/5xx） | 重试，最终记录为 failed，返回 `502/504` |
| `ParseError` | CSAF JSON 解析失败或格式不符合预期 | 记录日志，跳过该文件，继续处理其他 |
| `DatabaseError` | 数据库连接失败、查询错误 | 返回 `500 Internal Server Error`，记录日志 |
| `ConversionError` | CSAF 转 OVAL 时字段缺失或无法映射 | 记录日志，跳过该文件 |
| `IoError` | 文件读写错误（磁盘满、权限不足） | 返回 `500`，记录日志 |

### 11.2 全局错误响应格式

```json
{
  "error": "人类可读的错误描述",
  "code": "ERROR_CODE",
  "details": {  // 可选，携带额外上下文
    "field": "start",
    "value": "invalid"
  }
}
```

---

## 12. 附录：已确认设计决策汇总

本章汇总已确认的设计决策，供后续开发直接参考。

### 12.1 已确认的设计决策

| 决策项 | 确认结论 | 正文位置 |
|--------|----------|----------|
| ORM 选型 | **sqlx**（编译期检查，原生异步，多数据库兼容） | 第 3 章、第 6.4 节 |
| 多数据库支持 | **运行时切换**：`DbPool` 为 `enum { Sqlite, Postgres, Mysql }`，通过 `database.driver` 配置选择。三驱动默认编译，**无需重新编译**。7 个查询宏（`pool_exec!`、`pool_fetch!` 等）处理跨数据库差异（占位符、INSERT ID 获取方式） | 第 6.4 节 |
| 数据库存储 | 坚持**元数据拆分存储**，不存完整 XML 文本 | 第 5.3 节 |
| 合并去重 | 按 **OVAL ID** 对 tests/objects/states 去重 | 第 5.2.2 节、第 7.2.3 节 |
| 合并后命名 | 无需体现合并来源，generator 保持标准格式 | 第 5.2.2 节 |
| 增量同步 | 同名文件如果 `tracking.version` 更新，**覆盖本地记录** | 第 6.1 节 `sync` |
| CSAF 上传入库 | 新增 `POST /csaf` 接口（JSON / multipart 两种上传方式），与 sync 复用同一 `IngestService`，同步执行、单事务入库，支持 `dry_run` 预检与 `force` 重建 | 第 7.2.6 节 |
| `index_url` | 配置**父目录**地址（如 `https://.../advisories/`），自动拼接 `index.txt` 和文件名 | 第 6.1 节 `sync`、第 8 章 |
| CLI 命名 | 取**第一个完整软件包名**（不截断），如 `cuos-openssh-askpass-20251665.oval.xml` | 第 6.1 节 `convert` |
| 签名验证 | 服务模式：配置 `signature_keyid` 则生成检测，未配置则不生成；**CLI 始终不生成** | 第 6.3 节 |
| 平台检测 | 通过 `rpmverifyfile_test` 检测 `/etc/os-release`，支持 **CUOS** 和 **openEuler**，**必须精确版本**（VERSION_ID） | 第 6.3 节 |
| Epoch 获取 | 通过 **yum/dnf 源查询** epoch（`dnf repoquery --qf '%{EPOCH}'`），支持缓存，CLI 可通过参数手动指定 | 第 6.3 节 |
| Epoch 模块 | 在**独立文件或 crate 中实现**（`epoch-resolver`），对外暴露调用接口 | 第 6.3 节 |
| API 认证 | **JWT (Bearer Token)**，使用 `jsonwebtoken` crate，密码 bcrypt 哈希 | 第 13 章 |
| 用户管理 | 用户存 `users` 表（bcrypt 哈希）；**注册/禁用仅管理员经 CLI**（OS 层信任边界，免 admin 密码但记审计）；**改密双路径**——自助 `POST /auth/password`（验原密码）+ 管理员 CLI 重置（免原密码，置 `must_change_password` 强制改密）；首启经环境变量引导 admin，无默认口令 | 第 5.3、6.1、13.3.3、13.8 节 |
| 密钥轮换 | 主/备双 Secret + SIGHUP 热重载，轮换过程会话不中断；改单一 `jwt_secret` 需重载且旧 Token 全部失效 | 第 13.7、13.9 节 |
| 序列号设计 | **4 位序列号**（0001-9999），平台检测保留 0001-0100（支持多发行版/多版本），签名检测 0101-0200，版本检测 0201-9999 | 第 6.3 节 |
| XML 序列化 | **quick-xml + serde**：多命名空间用字面透传（`@xmlns:red-def` / `red-def:xxx` rename + 外部标签枚举）；美化用 `Serializer::indent(' ', 2)`；大合并用 `Writer::new_with_indent` 事件流 | 第 6.3 节"XML 序列化设计"（已 POC 实测） |

---

## 13. API 认证方案（JWT）

以下对比几种常见的 API 认证实现方式，供您选择。

### 方案对比

| 方案 | 原理 | 优点 | 缺点 | 适用场景 | 实现复杂度 |
|------|------|------|------|----------|----------|
| **API Key** | 每个调用方分配一个固定密钥字符串，通过 Header（如 `X-API-Key: abc123`）传递 | 实现最简单、无状态、易于分发和撤销 | 密钥泄露风险、无法携带用户信息、长期有效 | 内部服务、简单 B2B 对接、低安全要求 | ⭐ 低 |
| **JWT (Bearer Token)** | 服务端用私钥签发 JWT，客户端携带 `Authorization: Bearer <token>`，服务端验证签名和过期时间 | 无状态、可自包含用户信息、支持过期刷新、跨服务验证方便 | Token 一旦签发无法提前撤销（需黑名单）、Payload 大小有限 | 移动端、Web 前端、微服务间认证、当前**最主流**方案 | ⭐⭐ 中 |
| **OAuth 2.0 + OIDC** | 基于第三方授权服务器（如 Keycloak/Authing）签发 Access Token，支持授权码、客户端凭证等模式 | 标准化、支持 SSO、权限粒度可控、第三方集成能力强 | 架构复杂、依赖外部服务、过重 | 多应用统一认证、对外 SaaS 服务、需要 SSO 场景 | ⭐⭐⭐ 高 |
| **mTLS (双向 TLS)** | 服务端和客户端均使用 X.509 证书进行双向认证，在 TLS 握手阶段完成身份验证 | 安全性最高、传输层解决、无额外 Token 管理 | 证书管理复杂（CA、签发、轮换）、部署成本高 | 高安全要求（金融、政务）、内部基础设施通信 | ⭐⭐⭐ 高 |
| **Session + Cookie** | 服务端维护 Session 状态，通过 Cookie 传递 Session ID | 可立即撤销、服务端控制力强 | 有状态、需共享 Session（Redis 等）、不适合纯 API 调用 | 传统 Web 应用、后台管理系统 | ⭐⭐ 中 |

### 当前主流趋势

当前 REST API 认证的**主流选择**是：

1. **对外/多端场景（移动、Web、第三方）**：**JWT (Bearer Token)** + 可选 Refresh Token 机制。
   - 原因：无状态、跨平台、生态成熟（`jsonwebtoken` crate in Rust）。
2. **内部服务间通信**：**API Key** 或 **mTLS**。
   - 原因：简单直接，或安全要求极高。
3. **有统一身份管理需求**：**OAuth 2.0 + OIDC**（如接入企业统一认证平台）。

### Rust 生态推荐实现

| 方案 | Rust Crate | 说明 |
|------|-----------|------|
| API Key | 自定义中间件 | 读取 Header 与数据库/配置比对，actix-web 中间件实现简单 |
| JWT | **`jsonwebtoken`** | 最广泛使用的 JWT 库，支持 HS256/RS256 等算法，与 actix-web 集成成熟 |
| OAuth 2.0 | **`oauth2`** | 标准 OAuth 2.0 客户端/服务端实现，通常需配合外部 IdP（如 Keycloak） |
| mTLS | **`rustls`** / OpenSSL | actix-web 原生支持 `rustls` 配置客户端证书验证 |
| Session | **`actix-session`** | 支持 Redis/ Cookie 后端，适合 Web 后台管理 |

### 建议

结合 cu-scanner 的场景（提供 OVAL XML 下载，可能对接 `wget`/脚本），**推荐方案**：

| 阶段 | 方案 | 理由 |
|------|------|------|
| **当前版本** | **JWT**（Bearer Token） | 已确认采用，使用 `jsonwebtoken` crate 实现 |
| **后续版本** | 视需求扩展（如 OAuth 2.0 / mTLS） | 如有 SSO 或更高安全需求时考虑 |

---


### 13.1 方案概述

已确认采用 **JWT (Bearer Token)** 作为 API 认证方式。

### 13.2 认证流程

```
┌──────────────┐     POST /auth/login      ┌──────────────┐
│   客户端      │ ───────────────────────> │   服务端      │
│  (用户名/密码) │                         │  (签发 JWT)    │
│              │ <─── { token: "xxx" } ─── │              │
└──────────────┘                         └──────────────┘
       │
       │ 后续请求携带 Header
       │ Authorization: Bearer <token>
       ▼
┌──────────────┐     GET /oval/xxx         ┌──────────────┐
│   客户端      │ ── Authorization: Bearer──>│   服务端      │
│              │                         │  (验证 JWT)    │
│              │ <──── OVAL XML --------- │              │
└──────────────┘                         └──────────────┘
```

### 13.3 接口设计

#### 13.3.1 登录

```
POST /auth/login
Content-Type: application/json
```

**请求体**：
```json
{
  "username": "admin",
  "password": "xxx"
}
```

**成功响应** (`200 OK`):
```json
{
  "token": "eyJhbGciOiJIUzI1NiIs...",
  "token_type": "Bearer",
  "expires_in": 86400
}
```

**错误响应** (`401 Unauthorized`):
```json
{
  "error": "Invalid credentials",
  "code": "UNAUTHORIZED"
}
```

#### 13.3.2 Token 刷新

```
POST /auth/refresh
Authorization: Bearer <token>
```

**成功响应** (`200 OK`):
```json
{
  "token": "eyJhbGciOiJIUzI1NiIs...",
  "token_type": "Bearer",
  "expires_in": 86400
}
```

#### 13.3.3 用户自助修改密码

```
POST /auth/password
Authorization: Bearer <token>
Content-Type: application/json
```

**请求体**：
```json
{
  "old_password": "current-password",
  "new_password": "new-strong-password"
}
```

**处理逻辑**：
1. 校验 Bearer Token 有效；
2. 用 bcrypt 校验 `old_password` 与 `users.password_hash` 是否匹配，不匹配返回 `400 INVALID_OLD_PASSWORD`（统一话术，不泄露账号状态）；原密码错误计入 `failed_attempts`，受 5 次锁定约束；
3. 新密码执行口令策略（≥12 位、zxcvbn 弱口令检测、不与最近 5 次历史密码重复）；
4. 更新 `password_hash`，置 `must_change_password=false`、清零 `failed_attempts`；
5. 将**除当前 Token 外**该用户的历史 Token 加入黑名单（按 `jti`），防止旧 Token 继续持有已改密账号的访问权。

**成功响应** (`200 OK`)：
```json
{ "message": "password updated", "must_change_password": false }
```

**错误响应**：

| HTTP | code | 场景 |
|------|------|------|
| `400` | `INVALID_OLD_PASSWORD` | 原密码错误 |
| `400` | `WEAK_PASSWORD` | 新密码不符合口令策略（`details.reason` 说明原因） |
| `401` | `UNAUTHORIZED` | Token 无效或过期 |

> 管理员经 CLI `user passwd` 重置密码**不需要**原密码（CLI 信任边界为 OS 层，见 13.8 节权限模型表），重置后账号被置 `must_change_password=true`，用户下次登录必须经本接口完成改密。

### 13.4 JWT 配置

> **概念澄清**：配置文件中**不存在"静态 Token"**。`jwt_secret` 只是**签名密钥**，Token 是客户端调用 `/auth/login` 后由服务端**动态签发**的（含 `sub`/`jti`/`iat`/`exp`/`role`），每次登录产出的 Token 都不同。因此"更新 Token"是客户端行为（重新登录或调 `/auth/refresh`），与服务端配置无关；服务端需要管理的只有签名密钥，其轮换机制见 13.7 节。

```toml
[auth]
enabled = true
method = "jwt"
# 生产环境务必通过环境变量 CU_SCANNER_JWT_SECRET 注入，不要在配置文件中明文保存（见 9.5 节）
jwt_secret = "your-secret-key-here-min-32-bytes-long"
token_expire_hours = 24
# 可选：刷新 token 过期时间
refresh_token_expire_hours = 168
# 可选：Secret 轮换（见 13.7 节）。配置后签发使用 primary，验证依次尝试 primary/secondary；
# 未配置时回退到上面的 jwt_secret
# jwt_secret_primary = "new-secret"
# jwt_secret_secondary = "old-secret"
# 可选：Token 黑名单存储：memory / redis / database（见 13.7 节）
# token_blacklist = "memory"
```

### 13.5 Rust 实现

使用 `jsonwebtoken` crate：

```rust
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,        // 用户名
    pub jti: String,        // Token 唯一 ID（UUID），用于 9.9 节黑名单撤销
    pub iat: i64,           // 签发时间
    pub exp: i64,           // 过期时间
    pub role: String,       // 角色（admin/user）
}

pub struct JwtAuth {
    secret: String,
    expiration: i64,
}

impl JwtAuth {
    /// 签发 Token
    pub fn generate_token(&self, username: &str, role: &str) -> Result<String, Error>;
    
    /// 验证 Token
    pub fn verify_token(&self, token: &str) -> Result<Claims, Error>;
}

// actix-web 中间件
pub async fn jwt_middleware(
    req: ServiceRequest,
    auth: web::Data<JwtAuth>,
) -> Result<ServiceRequest, Error>;
```

### 13.6 安全要求

- `jwt_secret` 必须至少 32 字节，建议使用随机生成
- Token 过期时间默认 24 小时，可通过配置调整
- 支持 Refresh Token 机制（可选）
- 密码存储使用 bcrypt 哈希（`bcrypt` crate）
- 生产环境建议启用 HTTPS

### 13.7 Secret 轮换与 Token 撤销

**风险**：JWT Secret 长期不更换导致泄露后影响范围扩大；Token 被盗用后无法提前失效。

**防护措施**：
- **Secret 轮换**：
  - 支持配置主/备两个 Secret（`jwt_secret_primary` / `jwt_secret_secondary`），签发 Token 使用主 Secret，验证时依次尝试主/备 Secret。
  - 轮换流程：写入新主 Secret → 等待旧 Token 自然过期（如 24h）→ 移除旧 Secret。
  - 建议生产环境每 90 天轮换一次。
- **Token 黑名单（可选）**：
  - 对于需要立即撤销的场景（如用户离职、Token 泄露），将 Token 的 `jti`（JWT ID，见 13.5 节 Claims 定义）存入 Redis/数据库黑名单，有效期与 Token 过期时间一致。
  - 验证时先查黑名单，命中则拒绝。
  - 如无 Redis，可维护内存中的短期黑名单（LRU Cache，最大 10 万条）。
- **Token 绑定**：
  - Token 中携带签发时的 `iat`（签发时间），服务端可配置最小有效 `iat`（如 Secret 轮换后设置 `min_iat`，此前签发的 Token 全部失效）。

### 13.8 用户与凭证管理

**用户存储**：用户存于一期数据库 `users` 表（见 5.3 节），密码仅存 **bcrypt 哈希**（cost ≥ 12）。不引入外部 IdP；如未来接入企业统一认证（LDAP/OIDC），在 `[auth] method` 扩展即可，用户体系不变。

**管理方式（CLI，管理员操作）**：

```bash
cu-scanner user add --username ops01 --role user          # 交互式输入密码（不回显）
cu-scanner user list                                       # 列出用户（不显示哈希）
cu-scanner user passwd --username ops01                    # 管理员重置密码（置 must_change_password=true）
cu-scanner user disable --username ops01                   # 禁用（立即无法登录）
cu-scanner user enable  --username ops01                   # 启用
```

**权限模型（关键设计）**：

| 操作 | 执行方式 | 是否需要验证原密码/admin 密码 | 理由 |
|------|----------|------------------------------|------|
| 添加/禁用/启用用户、**管理员重置密码** | CLI（服务器本机执行） | **不需要** | CLI 的信任边界是 **OS 层**：能在服务器上执行 CLI 的人已拥有 shell 与数据库写权限（本可直接改库），再要求输入 admin 密码是"形式安全"，无实际收益。取而代之的是：命令必须用服务账号或 sudo 执行（OS 层控制），且**每次操作记录审计日志**（OS 用户名 `whoami`、时间、目标用户、操作类型） |
| **用户自助改密** | HTTP API `POST /auth/password`（见 13.3.3） | **需要原密码** + 有效 Bearer Token | 防止 Token 被盗用后直接改密锁死账号；原密码错误同样受 9.7 限流与失败锁定约束 |

**设计规则**：
1. **无自助注册 API**：本系统为内部工具，用户全部由管理员经 CLI 预置，攻击面最小化；
2. **密码修改双路径**：自助改密（API，验原密码）与管理员重置（CLI，免原密码但**强制** `must_change_password=true`）。`must_change_password=true` 的账号登录可成功，但除 `/auth/password` 外的所有接口返回 `403 PASSWORD_CHANGE_REQUIRED`，直至完成改密——避免管理员设置的初始/临时密码被长期使用；
3. **首次启动引导**：`users` 表为空且 `[auth] enabled = true` 时，从环境变量 `CU_SCANNER_ADMIN_USER` / `CU_SCANNER_ADMIN_PASSWORD` 创建初始 admin（同样置 `must_change_password=true`），未设置环境变量则拒绝启动认证相关路由并输出明显告警，**不存在默认口令**；
4. **角色模型**：`admin`（用户管理 + `POST /csaf` 等写操作 + 二期节点/扫描管理）/ `user`（只读查询 `/oval/*`）；鉴权中间件按路由所需角色校验 Claims；
5. **口令策略**：最小长度 12，zxcvbn 弱口令检测，禁止与最近 5 次历史密码重复；连续失败 5 次锁定 15 分钟（`failed_attempts` / `locked_until` 字段），与 IP 限流（9.7 节）形成双层防护；
6. **离职/泄露处置**：`user disable` + 13.7 节 Token 黑名单（按 `jti`）或 `min_iat` 整体失效，三层手段任选。

### 13.9 配置变更与服务重启的关系

> 回答常见疑问："更新密钥/配置是否必须重启服务？"

| 变更项 | 是否需要重启 | 说明 |
|--------|-------------|------|
| `jwt_secret`（单一密钥） | **需要重载**（SIGHUP 热重载或重启） | 重载后**此前签发的所有 Token 立即失效**（签名验证不过），客户端需重新登录。生产环境应避免直接改单一密钥，改用下方轮换流程 |
| 主/备 Secret 轮换（推荐） | 需要重载，但**会话不中断** | 流程：新密钥配为 `jwt_secret_primary`、旧密钥移到 `jwt_secret_secondary` → 重载 → 旧 Token 在过期前（≤24h）仍可用 secondary 验证通过 → 旧 Token 全部自然过期后移除 secondary 再重载。全程用户无感知 |
| 用户密码修改 / 用户禁用 | **无需重载** | 读写数据库即时生效；如需让该用户已签发的 Token 立即失效，走 13.7 黑名单（按 `jti`） |
| 普通业务配置（限流阈值、并发数等） | 视实现 | 建议支持 SIGHUP 热重载；不支持时重启，进程内无长事务，影响可控 |

**热重载实现**：监听 `SIGHUP`（Unix）重新读取 `config.toml` 与环境变量，原子替换内存中的配置快照（`arc-swap` / `RwLock<Config>`）；Windows 下无 SIGHUP，退化为重启或提供 `POST /api/v1/admin/reload`（仅 admin，可选）。

---

## 14. 性能与可靠性设计

### 14.1 性能设计

#### API 限流与并发控制

| 场景 | 策略 | 参数 |
|------|------|------|
| **API 限流** | 基于 `actix-web` 中间件或 `governor` crate 实现 Token Bucket 限流 | 默认 100 请求/分钟/IP，登录接口 5 请求/分钟，`POST /csaf` 上传接口 30 请求/分钟 |
| **数据库连接池** | `sqlx` 连接池动态管理 | `max_connections = 10`, `min_connections = 2`，支持连接超时 30s |
| **同步下载并发** | 限制同时下载的 CSAF 文件数 | `sync.concurrent_limit = 10` |
| **大 XML 合并** | 使用 `quick-xml` 流式写入，避免全部加载到内存 | 单次合并最大支持 5000 个 definition |

#### 单文件转换性能

CSAF → OVAL 的单文件转换是系统的核心路径，直接影响 CLI 批量转换效率和 `POST /csaf` 上传响应时间。以下为各场景的性能基线要求（以典型 CSAF 文件为基准：~15KB JSON，3 个 CVE，5 个 RPM 包）：

| 场景 | 性能指标 | 目标值 | 测量条件 |
|------|----------|--------|----------|
| **CLI 单文件转换**（epoch 缓存命中） | 端到端耗时 | **≤ 500ms** | 含 JSON 解析 → OVAL 转换 → XML 序列化 → 文件写入；epoch 从内存/DB 缓存命中 |
| **CLI 单文件转换**（epoch 缓存未命中） | 端到端耗时 | **≤ 5s** | epoch 需走 dnf repoquery 查询（受 `yum.timeout_sec` 约束）；含 dnf 子进程启动 + 查询时间 |
| **CLI 批量转换**（100 个文件，缓存预热后） | 平均单文件耗时 | **≤ 300ms** | epoch 全量预加载后批量转换，取平均 |
| **POST /csaf 上传入库**（epoch 缓存命中） | API 响应时间（P95） | **≤ 1.5s** | 含 JSON 解析 → 校验 → 转换 → 单事务写库（~10 张表）→ 返回 IngestResult |
| **POST /csaf 上传入库**（epoch 缓存未命中，dnf 查询路径） | API 响应时间（P95） | **≤ 6s** | 同上 + dnf epoch 查询；分母为服务模式持续运行中、dnf 子进程首次冷启动后的稳态调用 |
| **同步批量入库**（epoch 预热，DB 写入路径） | 单文件吞吐 | **≥ 5 文件/秒** | `sync.concurrent_limit = 10` 并发条件下，总吞吐 ≥ 50 文件/秒（含下载耗时波动，下载与转换并行流水线化） |

> **测量条件说明**：
> - 基准测试 CSAF 文件：~15KB，包含 3 个 CVE、5 个 RPM 包 dependency，生成约 30 个 OVAL 组件（1 def + 1 platform test + 5 version tests + 5 signature tests + 对应 objects/states）
> - "epoch 缓存命中"：`rpm_epoch_cache` 表中已有记录或同批次 `preload_epochs` 已预热
> - 文件写入包含磁盘 fsync 开销
> - P95 指 100 次重复请求中排序取第 95 百分位

#### CPU 与内存配置建议

基于上述性能要求，推荐以下硬件资源配置：

| 部署规模 | 场景 | 推荐 CPU | 推荐内存 | 说明 |
|----------|------|----------|----------|------|
| **最小配置（开发/测试）** | CLI 转换、本地调试 | 2 vCPU | 1 GB | 仅支持 SQLite、单文件 CLI 转换；不启用 epoch dnf 查询 |
| **标准配置（小规模生产）** | 服务模式，< 1000 个 definition，日均 < 100 次 API 调用 | 2 vCPU | 2 GB | 支持 SQLite/MySQL，epoch 缓存预热后运行；`sync.concurrent_limit ≤ 5` |
| **推荐配置（中等规模生产）** | 服务模式，1000–10000 个 definition，日均 100–1000 次 API 调用 | 4 vCPU | 4 GB | 推荐 MySQL/PostgreSQL；支持并发 sync、合并查询、epoch dnf 查询；`sync.concurrent_limit ≤ 10` |
| **高配（大规模生产）** | 服务模式，> 10000 个 definition，合并查询频繁，日均 > 1000 次调用 | 8 vCPU | 8 GB | MySQL/PostgreSQL 独立部署；启用 OVAL 合并结果缓存（LRU）；`sync.concurrent_limit ≤ 20` |

**CPU 选型说明**：

| CPU 代际 | 推荐型号示例 | 适用规模 | 备注 |
|----------|-------------|----------|------|
| x86_64（主流） | Intel Xeon Gold 64xx / AMD EPYC 7xxx 或同等 vCPU | 标准/推荐/高配 | Rust 编译产物为原生 x86_64 二进制，无虚拟机开销 |
| ARM64（鲲鹏/飞腾） | 华为鲲鹏 920 / 飞腾 S2500 | 标准/推荐 | `aarch64` target 交叉编译或原生编译；性能与 x86_64 基本持平 |
| 云主机（通用型） | 阿里云 ecs.g7 / 华为云 s6 / AWS m7g | 全部 | 按推荐 vCPU/内存 选择对应实例规格即可 |

**内存使用估算**（Rust release build，未开启jemalloc）：

| 内存消耗来源 | 典型占用 | 峰值场景 |
|-------------|----------|----------|
| 进程基础开销（actix-web + sqlx 连接池） | ~80 MB | 含 10 个 DB 连接 + tracing 缓冲区 |
| 单文件 CSAF→OVAL 转换 | +5 MB | JSON 解析 + OVAL 内存模型 + XML 序列化 String 缓冲区 |
| Epoch 内存缓存 | +2 MB | 按 2000 个包名 × ~1KB/条目 估算 |
| OVAL 合并缓存（可选 LRU） | +20 MB | 按 20 条缓存 × ~1MB/条（500 definitions 合并结果） |
| 同步批量处理（10 并发） | +50 MB | 10 个文件同时在内存中解析+转换 |
| **总计（推荐配置）** | **~150 MB 稳态** | **~250 MB 峰值** |

> **内存选型原则**：4 GB 配置中 Rust 进程常驻 ≤ 512 MB（留足操作系统 page cache 和 DB 缓冲池空间），剩余内存用于 OS 文件缓存加速 epoch repodata 解析和 OVAL XML 文件 I/O。

**CPU 核心分配建议**：

| 工作负载类型 | 推荐分配 | 说明 |
|-------------|----------|------|
| actix-web worker 线程 | `workers = 2–4`（等于物理核数） | 处理 HTTP 请求解析、JSON 序列化、参数校验等 CPU 密集型路径 |
| tokio 异步运行时 | 共用同一进程，由 tokio 调度 | reqwest 下载、sqlx DB I/O 均为异步等待，不占 CPU |
| dnf repoquery 子进程 | 短期 fork，不在主进程内 | 每次查询 fork 一个短生命周期进程，epoch 预热/缓存后极少触发 |
| sync 定时任务 | 与 API 请求共用 tokio runtime | 下载为 I/O 密集（几乎不占 CPU），转换与 DB 写入为 CPU/IO 混合，10 并发下 4 核足够 |

> **关键结论**：cu-scanner 是 **I/O 密集型**应用（HTTP 下载、数据库读写、XML 文件输出），而非 CPU 密集型。4 vCPU / 4 GB 即可覆盖绝大多数生产场景。如预期日均合并查询（`/oval/month` 或 `/oval/range`）非常频繁（> 500 次/天、每次合并 > 500 definition），建议升至 **8 vCPU / 8 GB** 并启用合并结果 LRU 缓存。

| 缓存类型 | 实现 | 说明 |
|----------|------|------|
| **Epoch 内存缓存** | `HashMap<String, String>` | 转换进程内缓存，避免重复查询 yum/dnf |
| **Epoch 数据库缓存** | `rpm_epoch_cache` 表 | 持久化缓存，epoch 值通常不变 |
| **OVAL 合并结果缓存**（可选） | 内存 LRU Cache | 缓存最近 20 次合并查询结果，TTL 5 分钟。**缓存键**为 `(查询类型, 月份/起止日期, 数据版本指纹)`，数据版本指纹取该范围内 `MAX(version)`+`COUNT(*)`；**sync 覆盖更新或新增入库后必须主动失效相关缓存**（或直接按指纹自动失效），避免返回过期数据 |

### 14.2 可靠性设计

#### 服务熔断与降级

| 场景 | 策略 |
|------|------|
| **数据库不可用时** | API 返回 `503 Service Unavailable`，同步任务暂停，每 30s 重试连接 |
| **CSAF 源不可用时** | 记录失败，保留上次成功同步的数据，继续提供 API 查询 |
| **yum/dnf epoch 查询失败** | 降级为默认值 `epoch = "0"`，记录 WARN 日志 |

#### 同步失败补偿机制

1. `download_tasks` 表记录每次下载的状态（`pending`/`success`/`failed`/`skipped`）。
2. 失败任务自动重试，最多 `sync.max_retries` 次，间隔指数退避（2s, 4s, 8s, 16s, 32s）。
3. 如果某次同步批次中部分文件失败，成功文件正常入库，失败文件记录到 `sync_batch_id` 下，下次同步优先处理。

#### 数据库迁移回滚策略

- 使用 `sqlx migrate` 管理迁移，每个迁移文件必须包含 `UP` 和 `DOWN` 脚本。
- 生产环境迁移前备份数据库。
- 记录迁移版本到 `__sqlx_migrations` 表，支持回滚到任意版本。

### 14.3 健康检查与监控

#### /health 接口增强

```json
{
  "status": "ok",
  "version": "1.0.0",
  "timestamp": "2025-12-01T10:00:00Z",
  "checks": {
    "database": { "status": "up", "latency_ms": 5 },
    "disk": { "status": "up", "free_gb": 45.2 },
    "last_sync": { "status": "up", "time": "2025-12-01T09:00:00Z" }
  }
}
```

#### 关键监控指标与告警阈值

| 指标 | 类型 | 告警阈值 | 说明 |
|------|------|----------|------|
| `api_request_duration_p99` | Gauge | > 2s | API P99 延迟 |
| `api_error_rate` | Gauge | > 1% | API 错误率 |
| `db_pool_active_connections` | Gauge | > 80% of max | 数据库连接池使用率 |
| `sync_last_success_timestamp` | Gauge | > 24h ago | 上次成功同步时间 |
| `sync_failed_count` | Counter | > 5 in 1h | 同步失败次数 |
| `oval_definitions_total` | Gauge | - | 总 OVAL 定义数 |
| `login_failure_rate` | Gauge | > 10 in 5min | 登录失败率（安全告警） |
| `conversion_duration_single_p95` | Histogram | > 1s（epoch 命中）/ > 6s（epoch 未命中） | 单文件 CSAF→OVAL 转换耗时 P95（CLI 和服务端分别打点，label: `mode=cli\|server`，`epoch=cached\|queried`）。对应 14.1 节单文件转换性能目标的实际运行值；超过阈值说明转换路径出现性能退化 |
| `conversion_throughput` | Gauge | < 5 files/s | 批量转换吞吐（仅 sync 路径统计：最近一个 `sync_batch_id` 中成功入库文件数 / 总耗时） |
| `process_resident_memory_mb` | Gauge | > 400 MB | Rust 进程 RSS，超过说明存在内存泄漏或未释放的大合并缓存（对照 14.1 节"内存使用估算"稳态 ~150 MB） |
| `epoch_resolve_duration_p95` | Histogram | > 10s | dnf repoquery epoch 查询耗时 P95；持续偏高说明 yum 源延迟过高或 dnf 缓存需要刷新，触发 epoch 降级使用默认值的频率上升 |

> **实现建议**：通过 `metrics` crate 暴露 Prometheus 格式指标，`/metrics` 接口无需认证（或仅内部网络可访问）。


