# 05 - Database

## Purpose

Database connection pool management, schema migrations, and repository implementations for all tables. Uses `sqlx` with compile-time checked queries.

## Dependencies

- 01-project-setup (dependencies)
- 02-config (DatabaseConfig)
- 03-error-types (AppError)
- 04-domain-models (DB row models)

## Files to Create

```
src/db/mod.rs
src/db/pool.rs
src/db/migrate.rs
src/db/repository/mod.rs
src/db/repository/csaf_source.rs
src/db/repository/oval_definition.rs
src/db/repository/oval_test.rs
src/db/repository/oval_object.rs
src/db/repository/oval_state.rs
src/db/repository/oval_criteria.rs
src/db/repository/oval_reference.rs
src/db/repository/oval_cve.rs
src/db/repository/oval_cpe.rs
src/db/repository/download_task.rs
src/db/repository/user.rs
src/db/repository/epoch_cache.rs

migrations/20250101000001_initial.sql   # All CREATE TABLE statements
migrations/20250101000002_indexes.sql   # All CREATE INDEX statements
```

## Reference Design Doc Sections

- Section 5.3 (Database table structures and indexes)
- Section 6.4 (Database module design)
- Section 9.11 (Database connection security)

---

## 5a. Connection Pool (`src/db/pool.rs`)

**Design**: `DbPool` is a 3-variant enum supporting runtime database selection via `database.driver` config field. No recompilation needed.

```rust
use crate::config::DatabaseConfig;
use crate::error::Result;
use sqlx::{SqlitePool, PgPool, MySqlPool};

#[derive(Clone)]
pub enum DbPool {
    Sqlite(SqlitePool),
    Postgres(PgPool),
    Mysql(MySqlPool),
}

impl DbPool {
    /// Create pool based on config.driver field.
    /// Supported drivers: "sqlite", "postgres" (or "pg"), "mysql"
    pub async fn create(config: &DatabaseConfig) -> Result<Self>;

    /// Begin a transaction (returns DbTransaction enum)
    pub async fn begin(&self) -> Result<DbTransaction<'_>>;
}

pub enum DbTransaction<'a> {
    Sqlite(sqlx::Transaction<'a, sqlx::Sqlite>),
    Postgres(sqlx::Transaction<'a, sqlx::Postgres>),
    Mysql(sqlx::Transaction<'a, sqlx::MySql>),
}

impl<'a> DbTransaction<'a> {
    pub async fn commit(self) -> Result<()>;
    pub async fn rollback(self) -> Result<()>;
}
```

**Query Macros** — wrap every sqlx call in a match on pool/transaction variant. Defined in `pool.rs` as `#[macro_export]`:

| Macro | Returns | Purpose |
|-------|---------|---------|
| `pool_exec!(pool, sql, params...)` | `Result<u64>` | INSERT/UPDATE/DELETE (returns affected rows) |
| `pool_insert_id!(pool, sql, params...)` | `Result<i64>` | INSERT returning new row ID |
| `pool_fetch!(pool, Type, sql, params...)` | `Result<Vec<Type>>` | SELECT many rows |
| `pool_fetch_opt!(pool, Type, sql, params...)` | `Result<Option<Type>>` | SELECT optional row |
| `pool_fetch_one!(pool, Type, sql, params...)` | `Result<Type>` | SELECT exactly one row |
| `tx_exec!(tx, sql, params...)` | `Result<u64>` | Transactional write |
| `tx_insert_id!(tx, sql, params...)` | `Result<i64>` | Transactional INSERT with ID |

**Driver differences handled by macros**:
- SQLite: `?` placeholders, `last_insert_rowid()` for ID
- PostgreSQL: `?` placeholders, `RETURNING id` for ID (auto-appended)
- MySQL: `?` placeholders, `last_insert_id()` for ID

**SQLite WAL mode** — automatically enabled on SQLite connections:
```sql
PRAGMA journal_mode=WAL;
PRAGMA busy_timeout=5000;
```

**Cargo.toml features** — all three drivers enabled by default:
```toml
[features]
default = ["sqlite", "postgres", "mysql"]
sqlite = ["sqlx/sqlite"]
postgres = ["sqlx/postgres"]
mysql = ["sqlx/mysql"]
```

---

## 5b. Migration Runner (`src/db/migrate.rs`)

```rust
/// Run all pending migrations against the pool.
/// Uses sqlx::migrate!() macro to embed migration files at compile time.
pub async fn run_migrations(pool: &AnyPool) -> Result<(), AppError>;
```

---

## 5c. Migrations (`migrations/`)

### `migrations/20250101000001_initial.sql`

Create all tables as specified in design doc Section 5.3. Use the exact column names, types, and constraints.

Key points:
- `csaf_sources.csaf_id` — `VARCHAR(64) UNIQUE NOT NULL`
- `oval_definitions.oval_id` + `version` — composite `UNIQUE(oval_id, version)`
- `oval_tests.oval_id` — `VARCHAR(128) UNIQUE NOT NULL`
- `oval_objects.oval_id` — `VARCHAR(128) UNIQUE NOT NULL`
- `oval_states.oval_id` — `VARCHAR(128) UNIQUE NOT NULL`
- `oval_criteria.parent_id` — self-referencing FK, nullable
- `rpm_epoch_cache.(pkg_name, repo_id)` — composite UNIQUE
- `users.username` — `VARCHAR(64) UNIQUE NOT NULL`
- All `TIMESTAMP` fields — store as UTC
- `SERIAL` for SQLite becomes `INTEGER PRIMARY KEY AUTOINCREMENT`

### `migrations/20250101000002_indexes.sql`

Create all indexes as specified in design doc Section 5.3 index table (19 indexes total).

---

## 5d. Repository Pattern

Each repository wraps `DbPool` and uses query macros for all database operations. All write operations that span multiple tables MUST be done within a single transaction.

### Repository Template

```rust
use crate::db::{DbPool, DbTransaction};
use crate::error::Result;

pub struct XxxRepository { pool: DbPool }

impl XxxRepository {
    pub fn new(pool: DbPool) -> Self { Self { pool } }

    // Reads: use pool_fetch! / pool_fetch_opt! macros
    pub async fn find_by_id(&self, id: i64) -> Result<Option<Row>> {
        pool_fetch_opt!(&self.pool, Row, "SELECT * FROM table WHERE id = ?", id)
    }

    // Writes in transaction: use tx_exec! / tx_insert_id! macros
    pub async fn insert(&self, tx: &mut DbTransaction<'_>, record: &InsertRecord) -> Result<i64> {
        tx_insert_id!(tx, "INSERT INTO table (col1, col2) VALUES (?, ?)", &record.col1, &record.col2)
    }

    pub async fn delete(&self, tx: &mut DbTransaction<'_>, id: i64) -> Result<()> {
        tx_exec!(tx, "DELETE FROM table WHERE id = ?", id)?;
        Ok(())
    }
}
```

**Macro imports**: Each repository file must import the macros it uses:
```rust
use crate::{pool_fetch, pool_fetch_opt, pool_insert_id, tx_exec, tx_insert_id};
```

### Repository: `CsafSourceRepository`

```rust
impl CsafSourceRepository {
    /// Find by csaf_id, returns None if not found
    pub async fn find_by_csaf_id(&self, csaf_id: &str) -> Result<Option<CsafSource>, AppError>;

    /// Find by oval_numeric_id
    pub async fn find_by_oval_numeric_id(&self, numeric_id: &str) -> Result<Option<CsafSource>, AppError>;

    /// Insert a new CSAF source record
    pub async fn insert(&self, tx: &mut Transaction<'_, Any>, record: &InsertCsafSource) -> Result<i64, AppError>;

    /// Update an existing record
    pub async fn update(&self, tx: &mut Transaction<'_, Any>, id: i64, record: &UpdateCsafSource) -> Result<(), AppError>;

    /// Query by date range (for merge queries)
    pub async fn find_by_date_range(&self, start: NaiveDate, end: NaiveDate) -> Result<Vec<CsafSource>, AppError>;
}
```

### Repository: `OvalDefinitionRepository`

```rust
impl OvalDefinitionRepository {
    /// Find by oval_id, returning the latest version
    pub async fn find_latest_by_oval_id(&self, oval_id: &str) -> Result<Option<OvalDefinitionRow>, AppError>;

    /// Find by csaf_id, returning the latest version
    pub async fn find_latest_by_csaf_id(&self, csaf_id: &str) -> Result<Option<OvalDefinitionRow>, AppError>;

    /// Query definitions by issued_date range, latest version per csaf_id
    pub async fn find_by_date_range(&self, start: NaiveDate, end: NaiveDate) -> Result<Vec<OvalDefinitionRow>, AppError>;

    /// Insert a new definition. Returns the auto-generated id.
    pub async fn insert(&self, tx: &mut Transaction<'_, Any>, def: &InsertOvalDefinition) -> Result<i64, AppError>;

    /// Delete all rows for a definition_id (used in update transaction)
    pub async fn delete_by_id(&self, tx: &mut Transaction<'_, Any>, id: i64) -> Result<(), AppError>;
}
```

### Repository: `OvalTestRepository`

```rust
impl OvalTestRepository {
    /// Insert a test row
    pub async fn insert(&self, tx: &mut Transaction<'_, Any>, test: &InsertOvalTest) -> Result<i64, AppError>;

    /// Find all tests for a definition
    pub async fn find_by_definition_id(&self, definition_id: i64) -> Result<Vec<OvalTestRow>, AppError>;

    /// Find all tests referenced by a list of definition_ids
    pub async fn find_by_definition_ids(&self, ids: &[i64]) -> Result<Vec<OvalTestRow>, AppError>;

    /// Delete all tests for a definition_id
    pub async fn delete_by_definition_id(&self, tx: &mut Transaction<'_, Any>, definition_id: i64) -> Result<(), AppError>;
}
```

### Repository: `OvalObjectRepository` / `OvalStateRepository` / `OvalCriteriaRepository`

Same pattern as `OvalTestRepository`: insert, find_by_definition_id, find_by_definition_ids, delete_by_definition_id.

### Repository: `OvalReferenceRepository` / `OvalCveRepository` / `OvalCpeRepository`

Same pattern: insert, find_by_definition_id, delete_by_definition_id.

**For merge queries** — need batch fetch by definition_ids:
```rust
impl OvalReferenceRepository {
    pub async fn find_by_definition_ids(&self, ids: &[i64]) -> Result<Vec<OvalReferenceRow>, AppError>;
}
impl OvalCveRepository {
    pub async fn find_by_definition_ids(&self, ids: &[i64]) -> Result<Vec<OvalCveRow>, AppError>;
}
impl OvalCpeRepository {
    pub async fn find_by_definition_ids(&self, ids: &[i64]) -> Result<Vec<OvalCpeRow>, AppError>;
}
```

### Repository: `DownloadTaskRepository`

```rust
impl DownloadTaskRepository {
    /// Find by file_name for sync dedup
    pub async fn find_by_file_name(&self, file_name: &str) -> Result<Option<DownloadTaskRow>, AppError>;

    /// Insert a new download task
    pub async fn insert(&self, task: &InsertDownloadTask) -> Result<i64, AppError>;

    /// Update task status
    pub async fn update_status(&self, id: i64, status: &str, error: Option<&str>) -> Result<(), AppError>;

    /// Find failed tasks from a sync batch
    pub async fn find_failed_by_batch(&self, batch_id: &str) -> Result<Vec<DownloadTaskRow>, AppError>;

    /// Delete records older than retention_days (default 730 = 2 years)
    pub async fn purge_old(&self, retention_days: i32) -> Result<u64, AppError>;
}
```

### Repository: `UserRepository`

```rust
impl UserRepository {
    /// Find by username
    pub async fn find_by_username(&self, username: &str) -> Result<Option<UserRow>, AppError>;

    /// List all users (for CLI `user list`)
    pub async fn list_all(&self) -> Result<Vec<UserRow>, AppError>;

    /// Insert a new user (password already bcrypt-hashed)
    pub async fn insert(&self, user: &InsertUser) -> Result<i64, AppError>;

    /// Update password hash
    pub async fn update_password(&self, id: i64, hash: &str) -> Result<(), AppError>;

    /// Update status (active/disabled)
    pub async fn update_status(&self, id: i64, status: &str) -> Result<(), AppError>;

    /// Update login tracking (last_login_at, failed_attempts, locked_until)
    pub async fn update_login_tracking(&self, id: i64, fields: &LoginTrackingUpdate) -> Result<(), AppError>;

    /// Reset failed_attempts and locked_until
    pub async fn reset_lockout(&self, id: i64) -> Result<(), AppError>;

    /// Count total users (for bootstrap check)
    pub async fn count(&self) -> Result<i64, AppError>;
}
```

### Repository: `EpochCacheRepository`

```rust
impl EpochCacheRepository {
    /// Find cached epoch by pkg_name and repo_id
    pub async fn find(&self, pkg_name: &str, repo_id: &str) -> Result<Option<EpochCacheRow>, AppError>;

    /// Insert or update cache entry (upsert)
    pub async fn upsert(&self, entry: &InsertEpochCache) -> Result<(), AppError>;
}
```

---

## Implementation Steps

1. Create migration SQL files matching the design doc Section 5.3 table definitions and indexes
2. Create `src/db/pool.rs`:
   - Implement `create_pool()` using `sqlx::any::AnyPoolOptions`
   - Handle SQLite WAL pragma
3. Create `src/db/migrate.rs`:
   - Use `sqlx::migrate!("migrations/")` macro
   - Call `.run(&pool).await`
4. Create `src/db/repository/mod.rs` — re-export all repositories
5. Implement each repository struct, one file per table
6. Write integration tests:
   - Create in-memory SQLite pool
   - Run migrations
   - Insert a CSAF source and query it back
   - Test the full transaction pattern (insert definition + tests + objects + states + criteria → verify all)
   - Test date-range queries
   - Test upsert behavior

## Transaction Pattern for Ingest

All writes during ingest (sync or POST /csaf) must happen in a single transaction:

```rust
async fn ingest_csaf(pool: &AnyPool, ...) -> Result<(), AppError> {
    let mut tx = pool.begin().await?;

    // 1. Upsert csaf_sources
    let csaf_id = csaf_repo.upsert(&mut tx, ...).await?;

    // 2. Insert oval_definitions
    let def_id = oval_def_repo.insert(&mut tx, ...).await?;

    // 3. Insert references, CVEs, CPEs
    for ref in &refs { oval_ref_repo.insert(&mut tx, def_id, ref).await?; }
    for cve in &cves { oval_cve_repo.insert(&mut tx, def_id, cve).await?; }
    for cpe in &cpes { oval_cpe_repo.insert(&mut tx, def_id, cpe).await?; }

    // 4. Insert tests, objects, states
    for t in &tests { oval_test_repo.insert(&mut tx, def_id, t).await?; }
    for o in &objects { oval_obj_repo.insert(&mut tx, o).await?; }
    for s in &states { oval_state_repo.insert(&mut tx, s).await?; }

    // 5. Insert criteria tree
    for c in &criteria { oval_criteria_repo.insert(&mut tx, def_id, c).await?; }

    tx.commit().await?;
    Ok(())
}
```

**Cover update**: When a CSAF with a newer `tracking.version` arrives:
1. Begin transaction
2. Update `csaf_sources` row (new tracking_version, release_date, etc.)
3. Delete old definition's associated rows (tests, objects, states, criteria, references, cves, cpes) by `definition_id`
4. Insert new definition + components
5. Commit

## Acceptance Criteria

- [ ] SQLite pool creates and applies WAL pragma automatically
- [ ] Migrations create all tables and indexes without errors
- [ ] Insert a CSAF source → `find_by_csaf_id` returns it
- [ ] Full transaction insert for a definition + all components → all components queryable
- [ ] Date range query returns correct definitions sorted by date
- [ ] `find_latest_by_oval_id` returns highest version when multiple versions exist
- [ ] Delete by definition_id removes all dependent rows
- [ ] MySQL connection string compiles with `mysql` feature
- [ ] PostgreSQL connection string compiles with `postgres` feature
