# 01 - Project Setup

## Purpose

Initialize the Rust project with all required dependencies, feature flags, and directory structure.

## Dependencies

None — this is the first module to implement.

## Files to Create

```
Cargo.toml
src/lib.rs
src/main.rs
```

## Cargo.toml Specification

```toml
[package]
name = "cu-scanner"
version = "1.0.0"
edition = "2021"
description = "CSAF to OVAL converter with HTTP API"

[dependencies]
# Web framework
actix-web = "4"
actix-cors = "0.7"
actix-rt = "2"

# Database
sqlx = { version = "0.8", features = ["runtime-tokio", "tls-rustls", "migrate"] }
# Database drivers enabled via features (see [features] below)
# sqlx-mysql, sqlx-postgres, sqlx-sqlite added per feature flag

# Serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"
quick-xml = { version = "0.38", features = ["serialize"] }

# HTTP client
reqwest = { version = "0.12", features = ["rustls-tls", "stream"], default-features = false }

# CLI
clap = { version = "4", features = ["derive", "env"] }

# Config
toml = "0.8"

# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "fmt", "json"] }
tracing-actix-web = "0.7"

# Auth
jsonwebtoken = "9"
bcrypt = "0.16"
uuid = { version = "1", features = ["v4"] }

# Time
chrono = { version = "0.4", features = ["serde"] }

# Async runtime
tokio = { version = "1", features = ["full"] }

# Cron scheduler (for server mode)
tokio-cron-scheduler = "0.13"

# Rate limiting
governor = "0.7"

# Metrics (optional, for /metrics endpoint)
metrics = "0.24"
metrics-exporter-prometheus = "0.16"

[dev-dependencies]
tempfile = "3"
wiremock = "0.6"       # Mock HTTP server for tests
rstest = "0.23"         # Parameterized tests

[features]
default = ["sqlite"]
mysql = ["sqlx/mysql"]
postgres = ["sqlx/postgres"]
sqlite = ["sqlx/sqlite"]
```

## src/lib.rs — Library Root

```rust
//! cu-scanner: CSAF to OVAL converter
//!
//! Converts CSAF JSON security advisories to OVAL XML patch definitions.
//! Supports CLI mode and Server mode with HTTP API.

pub mod config;
pub mod error;
pub mod models;
pub mod db;
pub mod parser;
pub mod epoch;
pub mod engine;
pub mod xml;
pub mod download;
pub mod ingest;
pub mod merge;
pub mod auth;
```

## src/main.rs — Binary Entry Point

```rust
use cu_scanner::cli;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    cli::run().await
}
```

## Feature Flag Matrix

| Feature | Adds | Use Case |
|---------|------|----------|
| `sqlite` (default) | `sqlx/sqlite` | Development, single-node deployment |
| `mysql` | `sqlx/mysql` | Production with MySQL |
| `postgres` | `sqlx/postgres` | Production with PostgreSQL |

## Implementation Steps

1. Run `cargo init cu-scanner` in the project root
2. Replace generated `Cargo.toml` with the one above
3. Create `src/lib.rs` with module declarations
4. Create `src/main.rs` with the tokio main entry
5. Run `cargo build` to verify all dependencies compile
6. Run `cargo test` to verify test harness works

## Acceptance Criteria

- [ ] `cargo build` succeeds with all dependencies
- [ ] `cargo test` compiles (even with no tests defined)
- [ ] `cargo build --no-default-features --features mysql` succeeds
- [ ] `cargo build --no-default-features --features postgres` succeeds
- [ ] `cargo check` produces no warnings
