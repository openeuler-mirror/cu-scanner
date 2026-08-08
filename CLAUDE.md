# CLAUDE.md

## Project Overview

**cu-scanner** is a Rust tool/service that converts CSAF (Common Security Advisory Framework) JSON security advisories into OVAL (Open Vulnerability and Assessment Language) XML patch definitions.

- **CLI mode**: `cu-scanner convert --input file.json --output out.xml` (single file or batch directory)
- **Server mode**: `cu-scanner server --config config.toml` (HTTP API with database persistence)
- **Sync mode**: `cu-scanner sync --config config.toml` (one-shot remote CSAF download + ingest)

## Build Commands

```bash
# Default: all three databases (SQLite, PostgreSQL, MySQL)
cargo build
cargo build --release

# With specific database backends only
cargo build --no-default-features --features sqlite
cargo build --no-default-features --features postgres
cargo build --no-default-features --features mysql

# Test
cargo test
cargo test -- --nocapture          # Show test output
cargo test --test integration_test  # Run specific test file

# Lint
cargo clippy -- -D warnings
cargo fmt --check

# Run (after build)
cargo run -- convert -i ./testdata/csaf-sample.json -o ./output.oval.xml
cargo run -- server -c config.toml
cargo run -- sync -c config.toml
cargo run -- user add --username admin --role admin
```

## Architecture

```
Entry Point (main.rs)
  → CLI (clap) → dispatch to:
      ├── convert  → Converter (CSAF→OVAL) → XML Serializer → file output
      ├── server   → actix-web Server (API + background cron sync)
      ├── sync     → Download Service + Ingest Service → DB
      └── user     → User Repository (CLI-only, no HTTP API)
```

**Core data flow**: `CSAF JSON → parse → validate → extract RPM packages/epoches → convert to OVAL structs → serialize to XML`

**Server data flow**: `HTTP request → JWT middleware → route handler → service layer → repository → DB → response (XML or JSON)`

**Sync data flow**: `Download index.txt → parse filenames → concurrent download CSAF files → each: parse → validate → convert → single-DB-transaction write`

## Source Directory Structure

```
src/
├── main.rs              # tokio::main entry point
├── lib.rs               # Library root (re-exports all modules)
├── config.rs            # TOML config loading (serde)
├── error.rs             # AppError enum + actix_web ResponseError impl
├── models/
│   ├── csaf.rs          # CSAF JSON input models (Deserialize)
│   ├── oval.rs          # OVAL XML output models (Serialize, quick-xml annotations)
│   └── db.rs            # Database row models (sqlx::FromRow)
├── db/
│   ├── pool.rs          # AnyPool creation, SQLite WAL pragma
│   ├── migrate.rs       # sqlx::migrate! runner
│   └── repository/      # One file per table, each wraps pool + CRUD
├── parser/
│   └── csaf.rs          # CSAF JSON → CsafDocument parsing + validation + extraction
├── epoch/
│   └── resolver.rs      # YumEpochResolver (dnf → repodata → default "0")
├── engine/
│   ├── id_gen.rs        # OVAL ID generator (4-digit sequence numbers)
│   └── converter.rs     # CsafDocument → OvalDefinitions conversion
├── xml/
│   └── serializer.rs    # OvalDefinitions → XML string/writer (quick-xml + serde)
├── download/
│   └── service.rs       # reqwest HTTP client, index.txt parsing, exponential backoff retry
├── ingest/
│   └── service.rs       # Pipeline: download→parse→convert→DB transaction
├── merge/
│   └── service.rs       # Query definitions by date, merge + dedup by OVAL ID
├── cli/
│   └── commands.rs      # clap derive: ConvertArgs, ServerArgs, SyncArgs, UserArgs
├── auth/
│   ├── jwt.rs           # jsonwebtoken encode/decode, Claims, blacklist
│   ├── middleware.rs    # actix-web JWT validator middleware
│   └── handlers.rs      # POST /auth/login, /auth/refresh, /auth/password
├── api/
│   ├── routes.rs        # Route configuration
│   ├── handlers.rs      # Health, /oval/{id}, /oval/month/{ym}, /oval/range, POST /csaf
│   ├── middleware.rs    # Rate limiter (governor), audit logger
│   └── response.rs      # JSON error formatting, security headers
└── server/
    └── app.rs           # Server startup: config→DB→services→actix-web→cron→signals
```

## Implementation Order

Modules MUST be implemented in this order; each depends only on earlier modules:

| Phase | Order | Module File | Depends On |
|-------|-------|-------------|------------|
| **Foundation** | 1 | `design/01-project-setup.md` | — |
| | 2 | `design/02-config.md` | 01 |
| | 3 | `design/03-error-types.md` | 01 |
| | 4 | `design/04-domain-models.md` | 01 |
| **Data** | 5 | `design/05-database.md` | 02, 03, 04 |
| **Parsing** | 6 | `design/06-csaf-parser.md` | 03, 04 |
| | 7 | `design/07-epoch-resolver.md` | 02, 03 |
| **Core** | 8 | `design/08-conversion-engine.md` | 03, 04, 06, 07 |
| | 9 | `design/09-xml-serializer.md` | 03, 04 |
| **Services** | 10 | `design/10-download-service.md` | 02, 03 |
| | 11 | `design/11-ingest-service.md` | 05, 06, 08, 10 |
| | 12 | `design/12-merge-service.md` | 05, 09 |
| **Interfaces** | 13 | `design/13-cli-module.md` | 02, 08, 09, 11, 12 |
| | 14 | `design/14-auth-module.md` | 02, 03, 05 |
| | 15 | `design/15-api-module.md` | 02, 03, 05, 11, 12, 14 |
| | 16 | `design/16-server-main.md` | All above |

**Implementation workflow**: For each module:
1. Read the corresponding `design/XX-module-name.md` for the full spec
2. Create the source file(s) listed in the module's "Files to Create" section
3. Implement the public API surface exactly as specified
4. Write tests for all test cases listed in the module spec
5. Verify all acceptance criteria are met before moving to the next module
6. Read the master design doc (`cu-scanner-design-doc.md`) for deeper context on any section referenced by the module

## Design Specs Reference

Detailed module specifications are in `design/`. Each file contains:
- **Public API** — complete fn signatures and struct definitions to implement
- **Dependencies** — which modules must be done first
- **Implementation details** — key algorithms and business logic
- **Test cases** — specific scenarios to test
- **Acceptance criteria** — checkable success conditions

The master design document is `cu-scanner-design-doc.md` (2235+ lines). Key sections:
- **Section 5.1**: CSAF JSON input structure
- **Section 5.2**: OVAL XML output structure (with namespace and ID conventions)
- **Section 5.3**: Database table schemas (12 tables, indexes)
- **Section 6.3**: Conversion engine details (ID generation, platform detection, signature verification, RPM parsing, XML serialization)
- **Section 7**: API interface design (routes, request/response formats)
- **Chapter 8**: TOML configuration format
- **Chapter 9**: Security design (SQL injection, XSS, CSRF, path traversal, JWT rotation)
- **Chapter 13**: JWT authentication (token format, login flow, password policy, secret rotation)
- **Chapter 14**: Performance requirements (single-file conversion targets, CPU/memory sizing)

## Key Conventions

### Error Handling
- **Every fallible function returns `Result<T, AppError>`** (defined in `src/error.rs`)
- Use `?` operator throughout — all external error types have `From` impls
- Never `.unwrap()` or `.expect()` in production code paths
- Error responses always use the format: `{"error": "...", "code": "ERROR_CODE", "details": {...}}`

### Database
- **All writes that span multiple tables MUST be in a single transaction**
- Use `sqlx` parameterized queries — **NEVER string-format SQL**
- **Runtime database selection** via `database.driver` config: `"sqlite"`, `"postgres"`, `"mysql"`. No recompilation needed.
- All `TIMESTAMP` columns store UTC; convert at presentation layer
- SQLite WAL mode auto-enabled: `PRAGMA journal_mode=WAL; PRAGMA busy_timeout=5000;`
- `DbPool` is an enum (`Sqlite`, `Postgres`, `Mysql`); `DbTransaction<'a>` is the corresponding transaction enum
- All repository queries use macros (`pool_fetch!`, `pool_exec!`, `tx_exec!`, `tx_insert_id!`, etc.) defined in `src/db/pool.rs`
- PostgreSQL INSERT-ID handling uses `RETURNING id` (auto-appended by `pool_insert_id!` macro)

### XML Serialization (quick-xml)
- **Namespace prefixes are literal strings** in serde annotations — typos won't be caught by the compiler
- Use `#[serde(rename = "@xmlns:red-def")]` for namespace declarations
- Use `#[serde(rename = "red-def:rpminfo_test")]` for prefixed elements
- Heterogeneous lists (tests/objects/states) use **external tag enums** (quick-xml doesn't support internal/adjacent tagging)
- Always validate serialized output against OVAL schema with `xmllint` in tests
- **CLI output**: pretty-printed (2-space indent) via `Serializer::indent(' ', 2)`
- **API output**: compact (no indent) for smaller payload size

### OVAL ID Conventions
```
oval:com.chinaunicom.cuos:{type}:{numeric_id}{seq}
  type: def | tst | obj | ste
  numeric_id: 8-digit from CSAF ID (CUOS-SA-2025-1665 → 20251665)
  seq: 4-digit zero-padded (0001-9999)
    0001-0100: platform detection
    0101-0200: signature verification
    0201-9999: version detection
```

### CSAF → OVAL Field Mapping
- `document.tracking.id` → `definition.id` (extract YYYYNNNN)
- `document.title` + aggregate_severity → `metadata.title` ("{csaf_id}: {title} ({severity})")
- `product_tree.branches` → `affected.platform` + `advisory.affected_cpe_list` (build CPE strings)
- `vulnerabilities[].product_status.fixed` → RPM version detection tests/objects/states
- Platform detection: `rpmverifyfile_test` on `/etc/os-release` (CUOS / openEuler)
- Signature check: `rpminfo_test` with signature_keyid (server mode only, requires config)
- Epoch: from `YumEpochResolver` (dnf → repodata → default "0")

### Service Mode vs CLI Mode
- **CLI mode** (`convert` subcommand): No signature verification tests generated. Epoch from `--epoch-map` or yum config. No database.
- **Server mode** (`server`/`sync`): Signature verification if `oval.enable_signature_check = true` AND `oval.signature_keyid` is set. Database required.

### Security
- **Passwords**: bcrypt hash (cost ≥ 12), never log or store cleartext
- **JWT**: HS256, secret ≥ 32 bytes, env var `CU_SCANNER_JWT_SECRET` preferred over config file
- **HTTPS**: enforce in production (TLS 1.2+), `reqwest` client must reject HTTP by default
- **Input validation**: All user input validated before use (regex, length limits, path canonicalization)
- **SQL injection**: Prevented by sqlx compile-time checked queries
- **Audit logging**: All auth operations, POST /csaf uploads, and sensitive queries logged with IP + user + timestamp

### RPM Filename Parsing
Parse from **right to left** (names can contain hyphens):
```
openssh-askpass-9.6p1-6.ule4.aarch64.rpm
└─ name ──────┘ └─ver─┘ └rel┘└─arch┘
```
1. Strip `.rpm` suffix
2. arch = last `-` segment
3. release = second-to-last `-` segment
4. version = third-to-last `-` segment
5. name = everything before the version

### Performance Targets
- CLI single-file conversion (epoch cached): **≤ 500ms**
- CLI single-file conversion (epoch miss, dnf query): **≤ 5s**
- CLI batch 100 files (preloaded epochs): **≤ 300ms avg per file**
- POST /csaf upload (epoch cached): **P95 ≤ 1.5s**
- Sync throughput (10 concurrent): **≥ 5 files/s per thread**
- Merge large range (1000 definitions): **≤ 10s**
- Maximum merge size: **5000 definitions** (hard cap)
- Process RSS memory: **~150 MB steady, ≤ 400 MB peak** (recommended 4 GB RAM)

## Testing Strategy

- **Unit tests**: Every public function in every module. Test happy path + edge cases.
- **Integration tests**: Database CRUD (in-memory SQLite), HTTP endpoints (actix_web::test), full conversion pipeline.
- **Snapshot tests**: Serialize OVAL → compare against expected XML files in `testdata/`.
- **Schema validation**: Run `xmllint --schema oval-definitions-schema.xsd` on serialized output.
- **Property tests** (optional): RPM filename parsing round-trip, ID generation uniqueness.

## Key Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| actix-web | 4 | HTTP framework |
| sqlx | 0.8 | Async database ORM (AnyPool for multi-DB) |
| quick-xml | 0.38 | XML serialization (with serde feature) |
| serde / serde_json | 1 | JSON/TOML serialization |
| clap | 4 | CLI argument parsing |
| reqwest | 0.12 | Async HTTP client |
| jsonwebtoken | 9 | JWT token handling |
| bcrypt | 0.16 | Password hashing |
| chrono | 0.4 | Date/time handling |
| tracing | 0.1 | Structured logging |
| tokio | 1 | Async runtime |
| tokio-cron-scheduler | 0.13 | Background sync scheduling |
| governor | 0.7 | Rate limiting |

## Files Never to Modify

- `cu-scanner-design-doc.md` — Master design document (read-only reference)
- `design/` directory files — Module specifications (read, implement from them, but only update if spec changes are explicitly requested)

## When Stuck

1. Re-read the module's `design/XX-module-name.md` spec
2. Search the master design doc (`cu-scanner-design-doc.md`) for the relevant section number
3. Check the error types in `src/error.rs` — most failures should map to an existing variant
4. Look at the database schema in `design/05-database.md` for table/column names
5. Check the domain models in `design/04-domain-models.md` for struct field names and serde annotations
