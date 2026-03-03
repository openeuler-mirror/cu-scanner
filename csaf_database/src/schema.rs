//! PostgreSQL数据库表结构定义
//!
//! 包含所有适用于PostgreSQL数据库的表创建语句。

/// 安全公告信息表创建语句 (PostgreSQL)
pub const CREATE_SA_INFO_TABLE_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS sa_info (
    id                SERIAL PRIMARY KEY,
    sa_id             TEXT NOT NULL UNIQUE,
    synopsis          TEXT,
    summary           TEXT,
    topic             TEXT,
    description       TEXT,
    severity          TEXT CHECK (severity IN ('NONE', 'LOW', 'MEDIUM', 'HIGH', 'CRITICAL')),
    affected_product  TEXT,
    affected_component TEXT,
    status            TEXT CHECK (status IN ('DRAFT', 'PUBLISHED', 'REVOKED', 'UPDATED')),
    created_time      TEXT DEFAULT (NOW() AT TIME ZONE 'UTC'),
    updated_time      TEXT DEFAULT (NOW() AT TIME ZONE 'UTC')
);
"#;

/// CVE信息表创建语句 (PostgreSQL)
pub const CREATE_CVE_INFO_TABLE_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS cve_info (
    id                SERIAL PRIMARY KEY,
    cve_id TEXT NOT NULL UNIQUE,
    description TEXT NOT NULL,
    base_severity TEXT CHECK (base_severity IN ('NONE', 'LOW', 'MEDIUM', 'HIGH', 'CRITICAL')),
    base_score REAL CHECK (base_score >= 0.0 AND base_score <= 10.0),
    vector_string TEXT,
    cvss_version TEXT,
    published_date TEXT,
    updated_date TEXT,
    status TEXT NOT NULL DEFAULT 'PUBLISHED' CHECK (status IN ('PUBLISHED', 'REJECTED')),
    created_at TEXT DEFAULT (NOW() AT TIME ZONE 'UTC'),
    updated_at TEXT DEFAULT (NOW() AT TIME ZONE 'UTC')
);
"#;

/// OS版本映射表创建语句 (PostgreSQL)
pub const CREATE_OS_VERSION_MAP_TABLE_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS os_version_map (
    id              SERIAL PRIMARY KEY,
    os_version      TEXT NOT NULL UNIQUE,
    upstream_series TEXT NOT NULL,
    dist            TEXT NOT NULL,
    release_date    TEXT,
    end_of_life     TEXT,
    description     TEXT
);
"#;

/// SA与CVE关联表创建语句 (PostgreSQL)
pub const CREATE_SA_CVE_TABLE_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS sa_cve (
    sa_id INTEGER NOT NULL,
    cve_id INTEGER NOT NULL,
    PRIMARY KEY (sa_id, cve_id),
    FOREIGN KEY (sa_id) REFERENCES sa_info(id) ON DELETE CASCADE,
    FOREIGN KEY (cve_id) REFERENCES cve_info(id) ON DELETE CASCADE
);
"#;

/// CVE影响信息表创建语句 (PostgreSQL)
pub const CREATE_CVE_AFFECT_TABLE_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS cve_affect (
    id INTEGER PRIMARY KEY,
    cve_id INTEGER NOT NULL,                  -- 关联 cve_info(id)
    package_name TEXT NOT NULL,               -- 包名
    os_version_id INTEGER NOT NULL,           -- 对应发行版
    status TEXT NOT NULL DEFAULT 'AFFECTED' CHECK (status IN ('AFFECTED', 'FIXED', 'UNAFFECTED', 'UNKNOWN')),
    fixed_version TEXT,                       -- 修复后版本号
    last_checked TEXT DEFAULT (NOW() AT TIME ZONE 'UTC'),
    FOREIGN KEY (cve_id) REFERENCES cve_info(id) ON DELETE CASCADE,
    FOREIGN KEY (os_version_id) REFERENCES os_version_map(id) ON DELETE CASCADE
);

-- 创建序列用于cve_affect表的id字段
CREATE SEQUENCE IF NOT EXISTS cve_affect_id_seq;
ALTER TABLE cve_affect ALTER COLUMN id SET DEFAULT nextval('cve_affect_id_seq');
ALTER SEQUENCE cve_affect_id_seq OWNED BY cve_affect.id;
"#;

/// 包源码映射表创建语句 (PostgreSQL)
pub const CREATE_PACKAGE_SOURCE_MAP_TABLE_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS package_source_map (
    id INTEGER PRIMARY KEY,
    package_name TEXT NOT NULL,
    os_version_id INTEGER NOT NULL,              -- 自有发行版
    upstream_series TEXT,                        -- 上游版本系列，如 "openEuler-24.03-LTS"
    is_inherited INTEGER DEFAULT 1,              -- 1=继承上游，0=自研
    created_at TEXT DEFAULT (NOW() AT TIME ZONE 'UTC'),
    updated_at TEXT DEFAULT (NOW() AT TIME ZONE 'UTC'),
    FOREIGN KEY (os_version_id) REFERENCES os_version_map(id)
);

-- 创建序列用于package_source_map表的id字段
CREATE SEQUENCE IF NOT EXISTS package_source_map_id_seq;
ALTER TABLE package_source_map ALTER COLUMN id SET DEFAULT nextval('package_source_map_id_seq');
ALTER SEQUENCE package_source_map_id_seq OWNED BY package_source_map.id;
"#;

/// 源码包信息表创建语句 (PostgreSQL)
pub const CREATE_SRC_RPM_INFO_TABLE_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS src_rpm_info (
    id INTEGER PRIMARY KEY,
    package_name TEXT NOT NULL, -- 源码包名
    version TEXT NOT NULL,
    release TEXT NOT NULL,
    dist TEXT,                  -- 对应发行版
    sa_id INTEGER,              -- 安全公告关联
    created_at TEXT DEFAULT (NOW() AT TIME ZONE 'UTC'),
    FOREIGN KEY (sa_id) REFERENCES sa_info(id)
);

-- 创建序列用于src_rpm_info表的id字段
CREATE SEQUENCE IF NOT EXISTS src_rpm_info_id_seq;
ALTER TABLE src_rpm_info ALTER COLUMN id SET DEFAULT nextval('src_rpm_info_id_seq');
ALTER SEQUENCE src_rpm_info_id_seq OWNED BY src_rpm_info.id;
"#;

/// 二进制包信息表创建语句 (PostgreSQL)
pub const CREATE_RPM_INFO_TABLE_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS rpm_info (
    id INTEGER PRIMARY KEY,
    package_name TEXT NOT NULL,
    version TEXT NOT NULL,
    release TEXT NOT NULL,
    dist TEXT,                  -- 对应发行版
    arch TEXT NOT NULL,
    src_rpm_id INTEGER,         -- 外键指向源码包
    created_at TEXT DEFAULT (NOW() AT TIME ZONE 'UTC'),
    FOREIGN KEY (src_rpm_id) REFERENCES src_rpm_info(id)
);

-- 创建序列用于rpm_info表的id字段
CREATE SEQUENCE IF NOT EXISTS rpm_info_id_seq;
ALTER TABLE rpm_info ALTER COLUMN id SET DEFAULT nextval('rpm_info_id_seq');
ALTER SEQUENCE rpm_info_id_seq OWNED BY rpm_info.id;
"#;

/// 已处理文件记录表创建语句 (PostgreSQL)
pub const CREATE_PROCESSED_FILE_TABLE_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS processed_file (
    id              SERIAL PRIMARY KEY,
    file_name       TEXT NOT NULL UNIQUE,
    file_type       TEXT,
    processed_time  TEXT DEFAULT (NOW() AT TIME ZONE 'UTC')
);

-- 创建序列用于processed_file表的id字段
CREATE SEQUENCE IF NOT EXISTS processed_file_id_seq;
ALTER TABLE processed_file ALTER COLUMN id SET DEFAULT nextval('processed_file_id_seq');
ALTER SEQUENCE processed_file_id_seq OWNED BY processed_file.id;
"#;

/// 创建所有数据库表的SQL语句 (PostgreSQL)
pub const CREATE_TABLES_SQL: &str = r#"
-- 安全公告信息表
CREATE TABLE IF NOT EXISTS sa_info (
    id                SERIAL PRIMARY KEY,
    sa_id             TEXT NOT NULL UNIQUE,
    synopsis          TEXT,
    summary           TEXT,
    topic             TEXT,
    description       TEXT,
    severity          TEXT CHECK (severity IN ('NONE', 'LOW', 'MEDIUM', 'HIGH', 'CRITICAL')),
    affected_product  TEXT,
    affected_component TEXT,
    status            TEXT CHECK (status IN ('DRAFT', 'PUBLISHED', 'REVOKED', 'UPDATED')),
    created_time      TEXT DEFAULT (NOW() AT TIME ZONE 'UTC'),
    updated_time      TEXT DEFAULT (NOW() AT TIME ZONE 'UTC')
);

-- CVE信息表
CREATE TABLE IF NOT EXISTS cve_info (
    id                SERIAL PRIMARY KEY,
    cve_id TEXT NOT NULL UNIQUE,
    description TEXT NOT NULL,
    base_severity TEXT CHECK (base_severity IN ('NONE', 'LOW', 'MEDIUM', 'HIGH', 'CRITICAL')),
    base_score REAL CHECK (base_score >= 0.0 AND base_score <= 10.0),
    vector_string TEXT,
    cvss_version TEXT,
    published_date TEXT,
    updated_date TEXT,
    status TEXT NOT NULL DEFAULT 'PUBLISHED' CHECK (status IN ('PUBLISHED', 'REJECTED')),
    created_at TEXT DEFAULT (NOW() AT TIME ZONE 'UTC'),
    updated_at TEXT DEFAULT (NOW() AT TIME ZONE 'UTC')
);

-- OS版本映射表
CREATE TABLE IF NOT EXISTS os_version_map (
    id              SERIAL PRIMARY KEY,
    os_version      TEXT NOT NULL UNIQUE,
    upstream_series TEXT NOT NULL,
    dist            TEXT NOT NULL,
    release_date    TEXT,
    end_of_life     TEXT,
    description     TEXT
);

-- SA与CVE关联表
CREATE TABLE IF NOT EXISTS sa_cve (
    sa_id INTEGER NOT NULL,
    cve_id INTEGER NOT NULL,
    PRIMARY KEY (sa_id, cve_id),
    FOREIGN KEY (sa_id) REFERENCES sa_info(id) ON DELETE CASCADE,
    FOREIGN KEY (cve_id) REFERENCES cve_info(id) ON DELETE CASCADE
);

-- CVE影响信息表
CREATE TABLE IF NOT EXISTS cve_affect (
    id INTEGER PRIMARY KEY,
    cve_id INTEGER NOT NULL,                  -- 关联 cve_info(id)
    package_name TEXT NOT NULL,               -- 包名
    os_version_id INTEGER NOT NULL,           -- 对应发行版
    status TEXT NOT NULL DEFAULT 'AFFECTED' CHECK (status IN ('AFFECTED', 'FIXED', 'UNAFFECTED', 'UNKNOWN')),
    fixed_version TEXT,                       -- 修复后版本号
    last_checked TEXT DEFAULT (NOW() AT TIME ZONE 'UTC'),
    FOREIGN KEY (cve_id) REFERENCES cve_info(id) ON DELETE CASCADE,
    FOREIGN KEY (os_version_id) REFERENCES os_version_map(id) ON DELETE CASCADE
);

-- 创建序列用于cve_affect表的id字段
CREATE SEQUENCE IF NOT EXISTS cve_affect_id_seq;
ALTER TABLE cve_affect ALTER COLUMN id SET DEFAULT nextval('cve_affect_id_seq');
ALTER SEQUENCE cve_affect_id_seq OWNED BY cve_affect.id;

-- 包源码映射表
CREATE TABLE IF NOT EXISTS package_source_map (
    id INTEGER PRIMARY KEY,
    package_name TEXT NOT NULL,
    os_version_id INTEGER NOT NULL,              -- 自有发行版
    upstream_series TEXT,                        -- 上游版本系列，如 "openEuler-24.03-LTS"
    is_inherited INTEGER DEFAULT 1,              -- 1=继承上游，0=自研
    created_at TEXT DEFAULT (NOW() AT TIME ZONE 'UTC'),
    updated_at TEXT DEFAULT (NOW() AT TIME ZONE 'UTC'),
    FOREIGN KEY (os_version_id) REFERENCES os_version_map(id)
);

-- 创建序列用于package_source_map表的id字段
CREATE SEQUENCE IF NOT EXISTS package_source_map_id_seq;
ALTER TABLE package_source_map ALTER COLUMN id SET DEFAULT nextval('package_source_map_id_seq');
ALTER SEQUENCE package_source_map_id_seq OWNED BY package_source_map.id;

-- 源码包信息表
CREATE TABLE IF NOT EXISTS src_rpm_info (
    id INTEGER PRIMARY KEY,
    package_name TEXT NOT NULL, -- 源码包名
    version TEXT NOT NULL,
    release TEXT NOT NULL,
    dist TEXT,                  -- 对应发行版
    sa_id INTEGER,              -- 安全公告关联
    created_at TEXT DEFAULT (NOW() AT TIME ZONE 'UTC'),
    FOREIGN KEY (sa_id) REFERENCES sa_info(id)
);

-- 创建序列用于src_rpm_info表的id字段
CREATE SEQUENCE IF NOT EXISTS src_rpm_info_id_seq;
ALTER TABLE src_rpm_info ALTER COLUMN id SET DEFAULT nextval('src_rpm_info_id_seq');
ALTER SEQUENCE src_rpm_info_id_seq OWNED BY src_rpm_info.id;

-- 二进制包信息表
CREATE TABLE IF NOT EXISTS rpm_info (
    id INTEGER PRIMARY KEY,
    package_name TEXT NOT NULL,
    version TEXT NOT NULL,
    release TEXT NOT NULL,
    dist TEXT,                  -- 对应发行版
    arch TEXT NOT NULL,
    src_rpm_id INTEGER,         -- 外键指向源码包
    created_at TEXT DEFAULT (NOW() AT TIME ZONE 'UTC'),
    FOREIGN KEY (src_rpm_id) REFERENCES src_rpm_info(id)
);

-- 创建序列用于rpm_info表的id字段
CREATE SEQUENCE IF NOT EXISTS rpm_info_id_seq;
ALTER TABLE rpm_info ALTER COLUMN id SET DEFAULT nextval('rpm_info_id_seq');
ALTER SEQUENCE rpm_info_id_seq OWNED BY rpm_info.id;

-- 已处理文件记录表
CREATE TABLE IF NOT EXISTS processed_file (
    id              SERIAL PRIMARY KEY,
    file_name       TEXT NOT NULL UNIQUE,
    file_type       TEXT,
    processed_time  TEXT DEFAULT (NOW() AT TIME ZONE 'UTC')
);

-- 创建序列用于processed_file表的id字段
CREATE SEQUENCE IF NOT EXISTS processed_file_id_seq;
ALTER TABLE processed_file ALTER COLUMN id SET DEFAULT nextval('processed_file_id_seq');
ALTER SEQUENCE processed_file_id_seq OWNED BY processed_file.id;
"#;

/// 获取所有表的创建顺序（考虑外键依赖关系）
pub const TABLE_CREATION_ORDER: &[&str] = &[
    "sa_info",
    "cve_info",
    "os_version_map",
    "sa_cve",
    "cve_affect",
    "package_source_map",
    "src_rpm_info",
    "rpm_info",
    "processed_file",
];

/// 安全公告信息表实体
#[derive(Debug, Clone)]
pub struct SaInfo {
    pub id: i32,
    pub sa_id: String,
    pub synopsis: Option<String>,
    pub summary: Option<String>,
    pub topic: Option<String>,
    pub description: Option<String>,
    pub severity: Option<String>,
    pub affected_product: Option<String>,
    pub affected_component: Option<String>,
    pub status: Option<String>,
    pub created_time: Option<String>,
    pub updated_time: Option<String>,
}

/// CVE信息表实体
#[derive(Debug, Clone)]
pub struct CveInfo {
    pub id: i32,
    pub cve_id: String,
    pub description: String,
    pub base_severity: Option<String>,
    pub base_score: Option<f32>,
    pub vector_string: Option<String>,
    pub cvss_version: Option<String>,
    pub published_date: Option<String>,
    pub updated_date: Option<String>,
    pub status: String,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

/// OS版本映射表实体
#[derive(Debug, Clone)]
pub struct OsVersionMap {
    pub id: i32,
    pub os_version: String,
    pub upstream_series: String,
    pub dist: String,
    pub release_date: Option<String>,
    pub end_of_life: Option<String>,
    pub description: Option<String>,
}

/// SA与CVE关联表实体
#[derive(Debug, Clone)]
pub struct SaCve {
    pub sa_id: i32,
    pub cve_id: i32,
}

/// CVE影响信息表实体
#[derive(Debug, Clone)]
pub struct CveAffect {
    pub id: i32,
    pub cve_id: i32,
    pub package_name: String,
    pub os_version_id: i32,
    pub status: String,
    pub fixed_version: Option<String>,
    pub last_checked: Option<String>,
}

/// 包源码映射表实体
#[derive(Debug, Clone)]
pub struct PackageSourceMap {
    pub id: i32,
    pub package_name: String,
    pub os_version_id: i32,
    pub upstream_series: Option<String>,
    pub is_inherited: i32,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

/// 源码包信息表实体
#[derive(Debug, Clone)]
pub struct SrcRpmInfo {
    pub id: i32,
    pub package_name: String,
    pub version: String,
    pub release: String,
    pub dist: Option<String>,
    pub sa_id: Option<i32>,
    pub created_at: Option<String>,
}

/// 二进制包信息表实体
#[derive(Debug, Clone)]
pub struct RpmInfo {
    pub id: i32,
    pub package_name: String,
    pub version: String,
    pub release: String,
    pub dist: Option<String>,
    pub arch: String,
    pub src_rpm_id: Option<i32>,
    pub created_at: Option<String>,
}

/// 已处理文件记录表实体
#[derive(Debug, Clone)]
pub struct ProcessedFile {
    pub id: i32,
    pub file_name: String,
    pub file_type: Option<String>,
    pub processed_time: Option<String>,
}
