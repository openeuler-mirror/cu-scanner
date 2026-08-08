# 07 - Epoch Resolver

## Purpose

Resolve RPM epoch values by querying local yum/dnf repositories, remote repodata, or falling back to default `"0"`. Results are cached in-memory and optionally in the database.

## Dependencies

- 02-config (YumConfig)
- 03-error-types
- 05-database (EpochCacheRepository)

## Files to Create

```
src/epoch/mod.rs
src/epoch/resolver.rs
```

## Reference Design Doc Sections

- Section 6.3 (RPM Epoch resolution, YumEpochResolver struct)
- Section 6.3 config example (yum section of config.toml)

## Public API Surface

```rust
// src/epoch/resolver.rs

use std::collections::HashMap;
use crate::config::YumConfig;
use crate::error::Result;

/// Resolves RPM epoch values from yum/dnf or repodata.
pub struct YumEpochResolver {
    /// In-memory cache: pkg_name → epoch
    cache: HashMap<String, String>,
    /// Database repository for persistent cache (if cache_to_db enabled)
    db_repo: Option<EpochCacheRepository>,
    /// Yum configuration
    config: YumConfig,
}

impl YumEpochResolver {
    /// Create a new resolver. If cache_to_db is enabled, requires a DB pool.
    pub fn new(config: YumConfig, db_repo: Option<EpochCacheRepository>) -> Self;

    /// Resolve epoch for a single package name.
    /// Priority: in-memory cache → DB cache → dnf query → repodata → default "0"
    pub async fn resolve_epoch(&mut self, pkg_name: &str) -> Result<String>;

    /// Batch preload epochs for multiple package names.
    /// Queries all at once via dnf/repoquery to minimize overhead.
    /// Populates the in-memory cache.
    pub async fn preload_epochs(&mut self, pkg_names: &[&str]) -> Result<()>;

    /// Check if epoch resolution is enabled
    pub fn is_enabled(&self) -> bool { self.config.enabled }

    /// Get epoch for a package (from cache only, no query).
    /// Returns None if not cached.
    pub fn get_cached(&self, pkg_name: &str) -> Option<&str>;
}
```

## Resolution Strategy (Priority Order)

| Priority | Method | Command/Source | Fallback on failure |
|----------|--------|---------------|---------------------|
| 1 | In-memory cache | `HashMap` lookup | → DB cache |
| 2 | DB cache | `rpm_epoch_cache` table lookup | → dnf query |
| 3 | Local dnf query | `dnf repoquery --qf '%{EPOCH}' <pkg>` | → repodata |
| 4 | Remote repodata | Download + parse `repodata/primary.xml.gz` | → default |
| 5 | Default | `"0"` | N/A |

## Key Implementation Details

### 1. DNF Query (Priority 3)

```rust
async fn query_dnf(&self, pkg_name: &str) -> Result<Option<String>> {
    let output = tokio::process::Command::new("dnf")
        .args(["repoquery", "--qf", "%{EPOCH}", pkg_name])
        .arg(if let Some(ref cfg) = self.config.repo_config {
            format!("--config={}", cfg)
        } else {
            String::new()
        })
        .output()
        .await?;

    if output.status.success() {
        let epoch = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if epoch.is_empty() || epoch == "(none)" {
            Ok(Some("0".to_string()))
        } else {
            Ok(Some(epoch))
        }
    } else {
        Ok(None)
    }
}
```

Timeout: wrap in `tokio::time::timeout(Duration::from_secs(config.timeout_sec))`.

### 2. Repodata Parsing (Priority 4)

If `yum.base_url` is configured, download `repodata/repomd.xml` to find the `primary.xml.gz` location. Download and parse it to find `<package><name>{pkg_name}</name><version epoch="..." .../></package>`.

Only implement this for the local-mode fallback. Remote repodata parsing can be a simplified XML scan using quick-xml Reader (not serde, since we only need the epoch attribute).

### 3. Caching

```rust
async fn cache_epoch(&mut self, pkg_name: &str, epoch: &str, source: &str) -> Result<()> {
    // Update in-memory cache
    self.cache.insert(pkg_name.to_string(), epoch.to_string());

    // Update DB cache if configured
    if let Some(ref repo) = self.db_repo {
        let repo_id = self.resolve_repo_id();
        repo.upsert(&InsertEpochCache {
            pkg_name: pkg_name.to_string(),
            repo_id,
            epoch: epoch.to_string(),
            source: source.to_string(),
            resolved_at: Some(Utc::now().naive_utc()),
        }).await?;
    }
    Ok(())
}
```

### 4. Batch Preload

For efficiency during scan/sync, preload all package epochs:

```rust
pub async fn preload_epochs(&mut self, pkg_names: &[&str]) -> Result<()> {
    if !self.config.enabled {
        // Set all to "0"
        for name in pkg_names {
            self.cache.insert(name.to_string(), "0".to_string());
        }
        return Ok(());
    }

    // Filter already-cached packages
    let uncached: Vec<&&str> = pkg_names.iter()
        .filter(|n| !self.cache.contains_key(**n))
        .collect();

    if uncached.is_empty() { return Ok(()); }

    // Try batch dnf query
    let pkgs_arg = uncached.iter().map(|n| **n).collect::<Vec<_>>().join(" ");
    match self.query_dnf_batch(&pkgs_arg).await {
        Ok(results) => {
            for (name, epoch) in results {
                self.cache_epoch(&name, &epoch, "dnf").await?;
            }
        }
        Err(_) => {
            // Fall back to individual queries or default
            for name in uncached {
                let epoch = self.resolve_epoch(name).await.unwrap_or_else(|_| "0".into());
                self.cache_epoch(name, &epoch, "default").await?;
            }
        }
    }
    Ok(())
}
```

## Error Handling

- DNF not found on system → log warning, use repodata or default
- Repodata unreachable → log warning, use default
- All methods failed → use `"0"` with warning
- Timeout during dnf query → cancel, fall to next method

**Key rule**: Epoch resolution should NEVER block the entire conversion pipeline. If all methods fail, silently use `"0"` and log a WARN.

## Test Cases

1. **Cache hit**: second call to `resolve_epoch("openssh")` returns cached value (no dnf call)
2. **Disabled**: `enabled = false` → all epochs return `"0"`
3. **Default fallback**: simulate dnf failure → returns `"0"`
4. **Preload fills cache**: call `preload_epochs(&["openssh", "bash"])` → both in cache afterward
5. **DB cache persistence**: after caching to DB, new resolver instance finds it in DB

## Acceptance Criteria

- [ ] `resolve_epoch` returns `"0"` when `enabled = false`
- [ ] In-memory cache prevents duplicate dnf calls
- [ ] DNF query timeout does not panic or block
- [ ] DB cache is checked before dnf query
- [ ] Fallback to default `"0"` when all methods fail (no error propagated)
- [ ] `preload_epochs` batch-queries efficiently
