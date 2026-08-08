# 11 - Ingest Service

## Purpose

Orchestrate the full pipeline: download CSAF → parse → validate → convert to OVAL → persist to database. This is the shared ingest logic used by both the sync subcommand (batch from index.txt) and the `POST /csaf` API endpoint (single file upload).

## Dependencies

- 02-config (SyncConfig, OvalConfig, YumConfig)
- 03-error-types
- 05-database (all repositories)
- 06-csaf-parser
- 07-epoch-resolver
- 08-conversion-engine (Converter, ConversionMode)
- 10-download-service

## Files to Create

```
src/ingest/mod.rs
src/ingest/service.rs
```

## Reference Design Doc Sections

- Section 6.1 (Sync subcommand behavior — version comparison)
- Section 7.2.6 (POST /csaf upload — shared logic)
- Section 5.3 (Transaction requirements)

## Public API Surface

```rust
// src/ingest/service.rs

use crate::error::Result;
use crate::models::csaf::CsafSummary;

/// Orchestrates the CSAF ingest pipeline.
/// Used by both sync (batch) and API upload (single).
pub struct IngestService {
    pool: AnyPool,
    converter: Converter,
    download_service: Option<DownloadService>, // None for API upload mode
    config: IngestConfig,
}

impl IngestService {
    pub fn new(
        pool: AnyPool,
        converter: Converter,
        download_service: Option<DownloadService>,
        config: IngestConfig,
    ) -> Self;

    /// Ingest a single CSAF from raw JSON bytes (API upload path).
    /// Returns the ingest result with action taken.
    pub async fn ingest_from_bytes(
        &self,
        data: &[u8],
        force: bool,
        dry_run: bool,
    ) -> Result<IngestResult>;

    /// Ingest from a download URL (sync path).
    /// Downloads the file, then calls ingest_from_bytes.
    pub async fn ingest_from_url(
        &self,
        base_url: &str,
        file_name: &str,
    ) -> Result<IngestResult>;

    /// Run full sync: fetch index, download new/changed files, ingest them.
    /// Returns a SyncReport with counts.
    pub async fn run_full_sync(&self) -> Result<SyncReport>;
}

/// Result of a single CSAF ingest operation
#[derive(Debug, Clone, Serialize)]
pub struct IngestResult {
    pub csaf_id: String,
    pub oval_id: String,
    pub tracking_version: String,
    pub action: IngestAction,
    pub severity: String,
    pub release_date: String,
    pub summary: IngestSummary,
    pub warnings: Vec<String>,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum IngestAction {
    Created,
    Updated,
    Skipped,       // Version unchanged
    DryRun,        // Validated only, not persisted
}

#[derive(Debug, Clone, Serialize)]
pub struct IngestSummary {
    pub packages: usize,
    pub cves: usize,
    pub tests: usize,
    pub objects: usize,
    pub states: usize,
}

/// Result of a full sync operation
#[derive(Debug, Clone)]
pub struct SyncReport {
    pub batch_id: String,
    pub total_files: usize,
    pub created: usize,
    pub updated: usize,
    pub skipped: usize,
    pub failed: usize,
    pub errors: Vec<SyncError>,
    pub duration_ms: u64,
}

#[derive(Debug, Clone)]
pub struct SyncError {
    pub file_name: String,
    pub error: String,
}

/// Configuration specifically for the ingest service
#[derive(Debug, Clone)]
pub struct IngestConfig {
    pub concurrent_limit: usize,
    pub max_retries: u32,
    pub retry_interval_sec: u64,
}
```

## Ingest Pipeline (Core Logic)

```
ingest_from_bytes(data, force, dry_run):
  1. Parse CSAF JSON from bytes
     └─ Invalid? → return 400 INVALID_JSON
  2. Validate CSAF structure
     └─ Invalid? → return 422 INVALID_CSAF with field list
  3. Extract csaf_id, tracking_version
  4. Query csaf_sources by csaf_id:
     ├─ Not found → action = Created
     ├─ Found, remote version > local → action = Updated
     ├─ Found, same version, force=false → return Skipped
     ├─ Found, same version, force=true → action = Updated
     └─ dry_run=true → action = DryRun (skip DB write)
  5. Extract packages, platforms, CVEs
  6. Resolve epochs for all packages
  7. Build CsafSummary for response
  8. Convert CSAF → OVAL (via Converter)
  9. If NOT dry_run:
     a. Begin DB transaction
     b. Upsert csaf_sources row
     c. If Updated: delete old definition components (tests/objects/states/criteria/refs/cves/cpes)
     d. Insert oval_definitions row
     e. Insert all components (tests, objects, states, criteria, references, cves, cpes)
     f. Commit transaction
  10. Return IngestResult
```

## Full Sync Flow

```
run_full_sync():
  1. Download index.txt from config.sync.index_url
  2. Parse filenames
  3. Generate batch_id = UUID
  4. For each filename (with concurrency limit):
     a. Create/update DownloadTask (pending)
     b. Download CSAF file bytes
     c. Call ingest_from_bytes()
     d. Update DownloadTask (success/failed)
     e. On success: emit "CSAF ingested" event
     f. On failure: log error, record in DownloadTask
  5. Return SyncReport
```

## Concurrency Control

```rust
use tokio::sync::Semaphore;
use std::sync::Arc;

async fn sync_with_concurrency(&self, files: Vec<String>) -> SyncReport {
    let semaphore = Arc::new(Semaphore::new(self.config.concurrent_limit));
    let mut handles = Vec::new();

    for file_name in files {
        let permit = semaphore.clone().acquire_owned().await.unwrap();
        let service = self.clone(); // or Arc-wrapped
        let base_url = self.config.index_url.clone();

        handles.push(tokio::spawn(async move {
            let _permit = permit;
            service.ingest_from_url(&base_url, &file_name).await
        }));
    }

    // Collect results
    let mut report = SyncReport::new();
    for handle in handles {
        match handle.await.unwrap() {
            Ok(result) => report.record(result),
            Err(e) => report.record_error(e),
        }
    }
    report
}
```

## Version Comparison

```rust
/// Compare two tracking versions (semver-ish strings like "1.0.0").
/// Returns Ordering::Greater if remote > local (needs update).
fn compare_versions(remote: &str, local: &str) -> std::cmp::Ordering {
    let r_parts: Vec<u32> = remote.split('.').filter_map(|s| s.parse().ok()).collect();
    let l_parts: Vec<u32> = local.split('.').filter_map(|s| s.parse().ok()).collect();
    r_parts.cmp(&l_parts)
}
```

## Test Cases

1. **New ingest**: CSAF not in DB → `action: Created`, all rows inserted
2. **Skip unchanged**: Same `tracking.version` → `action: Skipped`, no DB writes
3. **Force update**: Same version + `force=true` → `action: Updated`, components replaced
4. **Version update**: Newer version → `action: Updated`, old components deleted, new inserted
5. **Dry run**: `dry_run=true` → `action: DryRun`, no DB writes, summary returned
6. **Transaction rollback**: Insert fails midway → no partial data left in DB
7. **Epoch warning**: Epoch resolved to default→ warning included in result
8. **Concurrent limit**: 20 files, concurrent_limit=5 → max 5 simultaneous downloads

## Acceptance Criteria

- [ ] New CSAF file is fully ingested (source + definition + all components in DB)
- [ ] Duplicate version returns action=Skipped without DB writes
- [ ] Newer version triggers update (old components deleted, new inserted)
- [ ] Transaction ensures all-or-nothing writes
- [ ] Dry run validates and returns summary without writing
- [ ] Invalid CSAF returns proper error (not inserted)
- [ ] Sync report has accurate counts (created/updated/skipped/failed)
- [ ] Concurrency limit enforced during full sync
