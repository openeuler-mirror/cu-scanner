# 10 - Download Service

## Purpose

HTTP download service for CSAF files. Downloads `index.txt` file lists, parses filenames, downloads individual CSAF JSON files, with exponential backoff retry and concurrency control.

## Dependencies

- 02-config (SyncConfig)
- 03-error-types

## Files to Create

```
src/download/mod.rs
src/download/service.rs
```

## Reference Design Doc Sections

- Section 6.2 (Download Service module)
- Section 6.1 (Sync subcommand behavior)
- Section 9.12 (Download security)

## Public API Surface

```rust
// src/download/service.rs

use bytes::Bytes;
use reqwest::Client;
use crate::config::SyncConfig;
use crate::error::Result;

/// Service for downloading CSAF files from remote sources.
pub struct DownloadService {
    client: Client,
    config: SyncConfig,
}

impl DownloadService {
    /// Create a new download service with reqwest client configured
    /// with TLS, timeout, and user-agent.
    pub fn new(config: SyncConfig) -> Result<Self>;

    /// Download and parse an index.txt file from index_url.
    /// index_url must include the full URL to index.txt.
    /// Parses filenames, ignoring:
    ///   - Lines starting with '#' (comments)
    ///   - Empty lines
    ///   - Whitespace-only lines
    /// Returns a list of relative filenames (e.g., ["csaf-cuos-sa-2025-1665.json", ...])
    pub async fn fetch_index(&self, index_url: &str) -> Result<Vec<String>>;

    /// Download a single CSAF file.
    /// `base_url` is the parent directory URL (e.g., "https://.../advisories/").
    /// `file_name` is the relative filename from index.txt.
    /// Returns the raw bytes of the downloaded file.
    pub async fn download_file(&self, base_url: &str, file_name: &str) -> Result<Bytes>;
}

/// Data class for a parsed index entry
#[derive(Debug, Clone)]
pub struct IndexEntry {
    pub file_name: String,
    pub download_url: String,
}

/// Parse index.txt content into a list of filenames
pub fn parse_index_txt(content: &str) -> Vec<String>;
```

## Implementation Details

### reqwest Client Configuration

```rust
pub fn new(config: SyncConfig) -> Result<Self> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(config.timeout_sec))
        .user_agent("cu-scanner/1.0")
        .tls_built_in_root_certs(true)    // Use system CA bundle
        .https_only(true)                  // Reject HTTP (security requirement)
        .connect_timeout(Duration::from_secs(10))
        .build()
        .map_err(|e| AppError::Internal {
            message: format!("Failed to create HTTP client: {}", e),
        })?;

    Ok(Self { client, config })
}
```

### index.txt Parsing

```rust
pub fn parse_index_txt(content: &str) -> Vec<String> {
    content
        .lines()
        .map(|line| line.trim().to_string())
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .collect()
}
```

### Download with Retry

```rust
pub async fn download_file(&self, base_url: &str, file_name: &str) -> Result<Bytes> {
    let url = format!("{}{}", base_url.trim_end_matches('/'), file_name);
    let max_retries = self.config.max_retries;
    let base_interval = Duration::from_secs(self.config.retry_interval_sec);

    let mut last_error = None;

    for attempt in 0..=max_retries {
        if attempt > 0 {
            // Exponential backoff: 2s, 4s, 8s, 16s, 32s
            let delay = base_interval * 2u64.pow(attempt as u32 - 1);
            tokio::time::sleep(delay).await;
        }

        match self.client.get(&url).send().await {
            Ok(response) => {
                // Check status code
                let status = response.status();
                if status.is_success() {
                    return response.bytes().await.map_err(|e| AppError::Download {
                        url: url.clone(),
                        message: format!("Failed to read response body: {}", e),
                        status_code: Some(status.as_u16()),
                    });
                }

                // Don't retry 4xx client errors (except 429)
                if status.is_client_error() && status.as_u16() != 429 {
                    return Err(AppError::Download {
                        url: url.clone(),
                        message: format!("Client error: HTTP {}", status),
                        status_code: Some(status.as_u16()),
                    });
                }

                last_error = Some(AppError::Download {
                    url: url.clone(),
                    message: format!("Server error: HTTP {}", status),
                    status_code: Some(status.as_u16()),
                });
            }
            Err(e) => {
                last_error = Some(AppError::Download {
                    url: url.clone(),
                    message: format!("Request failed: {}", e),
                    status_code: None,
                });
            }
        }
    }

    Err(last_error.unwrap_or_else(|| AppError::Download {
        url: url.clone(),
        message: "Max retries exceeded".into(),
        status_code: None,
    }))
}
```

### Response Size Limit

Enforce a download size limit (default 50MB) to prevent memory exhaustion:

```rust
// In download_file, after receiving response:
let content_length = response.content_length().unwrap_or(0);
if content_length > MAX_DOWNLOAD_SIZE {
    return Err(AppError::Download {
        url: url.clone(),
        message: format!("Response too large: {} bytes (max: {})", content_length, MAX_DOWNLOAD_SIZE),
        status_code: None,
    });
}

const MAX_DOWNLOAD_SIZE: u64 = 50 * 1024 * 1024; // 50 MB
```

## Test Cases

1. **index.txt parsing**: `"file1.json\n# comment\nfile2.json\n"` → `["file1.json", "file2.json"]`
2. **Empty index**: Empty string → empty vec
3. **URL construction**: `base_url="https://example.com/advisories/"`, `file_name="csaf-1.json"` → `"https://example.com/advisories/csaf-1.json"`
4. **HTTPS enforcement**: HTTP URL → client rejects (or test that builder rejects)
5. **Retry on 503**: Mock server returns 503 twice, then 200 → succeeds on third attempt
6. **No retry on 404**: Mock server returns 404 → returns error immediately (no retries)
7. **Size limit**: Response with Content-Length > 50MB → returns error
8. **Full integration**: fetch_index + download_file against a mock server

## Acceptance Criteria

- [ ] `parse_index_txt` correctly filters comments and empty lines
- [ ] Download retries on 5xx with exponential backoff
- [ ] Download does NOT retry on 4xx (except 429)
- [ ] HTTPS is enforced (HTTP URLs rejected)
- [ ] Response size > 50MB returns error before downloading
- [ ] Timeout triggers after `sync.timeout_sec` seconds
- [ ] `fetch_index` constructs correct full URL and parses result
