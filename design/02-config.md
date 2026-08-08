# 02 - Configuration

## Purpose

Define configuration structures that map to `config.toml`, load them at startup, and support environment variable overrides.

## Dependencies

- 01-project-setup

## Files to Create

```
src/config.rs
```

## Reference Design Doc Sections

- Chapter 8 (Configuration Design)
- Chapter 13.4 (JWT Configuration)

## Public API Surface

```rust
// src/config.rs

use serde::Deserialize;
use std::path::Path;

/// Top-level configuration, maps 1:1 to config.toml
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub logging: LoggingConfig,
    pub sync: SyncConfig,
    pub yum: YumConfig,
    pub oval: OvalConfig,
    pub auth: AuthConfig,
}

// --- Sub-config sections ---

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    #[serde(default = "default_host")]
    pub host: String,                         // default: "127.0.0.1"
    #[serde(default = "default_port")]
    pub port: u16,                            // default: 8080
    #[serde(default = "default_workers")]
    pub workers: usize,                       // default: 4
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    pub driver: String,                       // "sqlite" | "mysql" | "postgres"
    pub url: String,                          // connection string (env var override: DATABASE_URL)
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,                 // default: 10
    #[serde(default = "default_min_connections")]
    pub min_connections: u32,                 // default: 2
}

#[derive(Debug, Clone, Deserialize)]
pub struct LoggingConfig {
    #[serde(default = "default_log_level")]
    pub level: String,                        // default: "INFO"
    #[serde(default = "default_log_format")]
    pub format: String,                       // default: "text"
    pub file: Option<String>,                 // log file path
    #[serde(default = "default_true")]
    pub console: bool,                        // default: true
}

#[derive(Debug, Clone, Deserialize)]
pub struct SyncConfig {
    pub index_url: String,                    // e.g. "https://www.chinaunicom.com/security/advisories/"
    #[serde(default)]
    pub on_startup: bool,                     // default: false
    pub cron: Option<String>,                 // e.g. "0 0 */6 * * *" (6-field, sec first)
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,                     // default: 5
    #[serde(default = "default_retry_interval")]
    pub retry_interval_sec: u64,              // default: 2
    #[serde(default = "default_concurrent_limit")]
    pub concurrent_limit: usize,              // default: 10
    #[serde(default = "default_timeout")]
    pub timeout_sec: u64,                     // default: 30
}

#[derive(Debug, Clone, Deserialize)]
pub struct YumConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,                        // default: true
    pub repo_config: Option<String>,          // e.g. "/etc/yum.repos.d/cuos.repo"
    pub base_url: Option<String>,             // e.g. "https://repo.cucloud.com/cuos/4.0/os/"
    #[serde(default = "default_timeout")]
    pub timeout_sec: u64,                     // default: 30
    #[serde(default = "default_true")]
    pub cache_to_db: bool,                    // default: true
}

#[derive(Debug, Clone, Deserialize)]
pub struct OvalConfig {
    #[serde(default = "default_namespace")]
    pub namespace: String,                    // default: "com.chinaunicom.cuos"
    #[serde(default = "default_schema_version")]
    pub schema_version: String,               // default: "5.10"
    #[serde(default = "default_product_name")]
    pub product_name: String,                 // default: "cu-scanner"
    #[serde(default = "default_advisory_from")]
    pub advisory_from: String,                // default: "security@chinaunicom.com"
    #[serde(default = "default_rights")]
    pub rights: String,                       // default: "Copyright 2025 ChinaUnicom, Inc."
    #[serde(default)]
    pub enable_signature_check: bool,         // default: false
    pub signature_keyid: Option<String>,      // e.g. "199e2f91fd431d51"
}

#[derive(Debug, Clone, Deserialize)]
pub struct AuthConfig {
    #[serde(default)]
    pub enabled: bool,                        // default: false
    #[serde(default = "default_auth_method")]
    pub method: String,                       // default: "jwt"
    pub jwt_secret: Option<String>,           // env override: CU_SCANNER_JWT_SECRET
    pub jwt_secret_primary: Option<String>,
    pub jwt_secret_secondary: Option<String>,
    #[serde(default = "default_token_expire")]
    pub token_expire_hours: u64,              // default: 24
    #[serde(default = "default_refresh_expire")]
    pub refresh_token_expire_hours: u64,      // default: 168
    #[serde(default = "default_blacklist")]
    pub token_blacklist: String,              // default: "memory" (memory | redis | database)
}

// --- Default value functions ---
fn default_host() -> String { "127.0.0.1".into() }
fn default_port() -> u16 { 8080 }
fn default_workers() -> usize { 4 }
fn default_max_connections() -> u32 { 10 }
fn default_min_connections() -> u32 { 2 }
fn default_log_level() -> String { "INFO".into() }
fn default_log_format() -> String { "text".into() }
fn default_true() -> bool { true }
fn default_max_retries() -> u32 { 5 }
fn default_retry_interval() -> u64 { 2 }
fn default_concurrent_limit() -> usize { 10 }
fn default_timeout() -> u64 { 30 }
fn default_namespace() -> String { "com.chinaunicom.cuos".into() }
fn default_schema_version() -> String { "5.10".into() }
fn default_product_name() -> String { "cu-scanner".into() }
fn default_advisory_from() -> String { "security@chinaunicom.com".into() }
fn default_rights() -> String { "Copyright 2025 ChinaUnicom, Inc.".into() }
fn default_auth_method() -> String { "jwt".into() }
fn default_token_expire() -> u64 { 24 }
fn default_refresh_expire() -> u64 { 168 }
fn default_blacklist() -> String { "memory".into() }

// --- Config loading ---

impl Config {
    /// Load config from a TOML file path.
    /// Environment variables override file values:
    ///   CU_SCANNER_DATABASE_URL  → database.url
    ///   CU_SCANNER_JWT_SECRET    → auth.jwt_secret
    ///   CU_SCANNER_ADMIN_USER    → bootstrap admin username
    ///   CU_SCANNER_ADMIN_PASSWORD → bootstrap admin password
    pub fn load(path: &Path) -> Result<Self, ConfigError>;

    /// Resolve secrets: check env vars, validate required fields
    pub fn resolve_secrets(&mut self);
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Failed to read config file: {0}")]
    Io(#[from] std::io::Error),
    #[error("Failed to parse TOML: {0}")]
    Parse(#[from] toml::de::Error),
    #[error("Required config field missing: {0}")]
    MissingField(String),
    #[error("Invalid value for {field}: {message}")]
    InvalidValue { field: String, message: String },
}
```

## Implementation Steps

1. Define all config structs with serde Deserialize derive
2. Implement all `default_*()` functions so every field has a reasonable default
3. Implement `Config::load(path)`:
   - Read file content with `std::fs::read_to_string`
   - Parse TOML with `toml::from_str`
   - Call `resolve_secrets()` to apply env var overrides
4. Implement `Config::resolve_secrets()`:
   - Check `CU_SCANNER_DATABASE_URL` env var → override `database.url`
   - Check `CU_SCANNER_JWT_SECRET` env var → override `auth.jwt_secret`
   - Validate: if `auth.enabled = true` and no JWT secret, return error
   - Validate: `database.url` is not empty
5. Add `Config` as actix-web application data (wrapped in `Arc<RwLock<Config>>` for hot-reload)
6. Write unit tests for default values and env var overrides

## Key Rules

- **Every field has a default** — the only truly required field is `database.url` (or `DATABASE_URL` env var)
- **Environment variables take precedence** over TOML file values
- **Passwords/secrets must NOT be logged** — implement `Debug` manually or use `#[serde(skip)]` to redact
- **Config is reloadable** via SIGHUP — wrap in `Arc<RwLock<Config>>` for the server

## Acceptance Criteria

- [ ] `Config::load("config.toml")` parses the example config from the design doc
- [ ] Empty config file fills all defaults
- [ ] `DATABASE_URL` env var overrides `database.url`
- [ ] `CU_SCANNER_JWT_SECRET` env var overrides `auth.jwt_secret`
- [ ] Missing database URL returns `ConfigError::MissingField`
- [ ] Auth enabled without secret returns `ConfigError::MissingField`
- [ ] Debug output of Config does NOT show passwords or secrets
