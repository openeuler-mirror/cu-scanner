# 03 - Error Types

## Purpose

Define unified error types used throughout the application. Every module returns `Result<T, AppError>`.

## Dependencies

- 01-project-setup

## Files to Create

```
src/error.rs
```

## Reference Design Doc Sections

- Chapter 11 (Error Handling)

## Public API Surface

```rust
// src/error.rs

use actix_web::{HttpResponse, ResponseError};
use std::fmt;

/// Primary application error type.
/// All fallible functions in the codebase return `Result<T, AppError>`.
#[derive(Debug)]
pub enum AppError {
    // --- Validation (4xx) ---
    /// Input parameter validation failed (date format, path, etc.)
    Validation { message: String, field: Option<String> },
    /// CSAF JSON structure validation failed
    CsafValidation { message: String, fields: Vec<String> },
    /// Request payload too large (>10MB)
    PayloadTooLarge { limit_bytes: u64, actual_bytes: u64 },
    /// Resource not found (OVAL ID, month query with no results)
    NotFound { resource: String, id: String },
    /// Date range exceeds max allowed span (365 days)
    RangeTooLarge { start: String, end: String, max_days: u32 },
    /// Concurrent ingest conflict on same csaf_id
    Conflict { csaf_id: String },
    /// Authentication/authorization failure
    Unauthorized { message: String },
    /// Password change required
    PasswordChangeRequired,

    // --- Infrastructure (5xx) ---
    /// Database operation failed
    Database { message: String, source: Option<Box<dyn std::error::Error + Send + Sync>> },
    /// HTTP download failed
    Download { url: String, message: String, status_code: Option<u16> },
    /// CSAF JSON parse error
    Parse { message: String, file: Option<String> },
    /// CSAF → OVAL conversion error (missing fields, unhandled case)
    Conversion { message: String, csaf_id: Option<String> },
    /// File I/O error
    Io { message: String, source: std::io::Error },
    /// Configuration error
    Config { message: String },
    /// Internal / unexpected error
    Internal { message: String },

    // --- External error wrappers ---
    /// Wraps errors from external crates
    External { message: String, source: Box<dyn std::error::Error + Send + Sync> },
}

// --- Conversions from common error types ---

impl From<std::io::Error> for AppError { /* → AppError::Io */ }
impl From<sqlx::Error> for AppError { /* → AppError::Database */ }
impl From<serde_json::Error> for AppError { /* → AppError::Parse */ }
impl From<reqwest::Error> for AppError { /* → AppError::Download */ }
impl From<toml::de::Error> for AppError { /* → AppError::Config */ }
impl From<chrono::ParseError> for AppError { /* → AppError::Validation */ }
impl From<quick_xml::Error> for AppError { /* → AppError::Internal */ }
impl From<jsonwebtoken::errors::Error> for AppError { /* → AppError::Unauthorized */ }

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Print the human-readable message
        match self {
            // ... match each variant
        }
    }
}

impl std::error::Error for AppError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None // simplified; Database/Io variants can return source
    }
}

// --- actix-web integration ---

impl ResponseError for AppError {
    fn status_code(&self) -> actix_web::http::StatusCode {
        match self {
            AppError::Validation { .. }     => StatusCode::BAD_REQUEST,
            AppError::CsafValidation { .. } => StatusCode::UNPROCESSABLE_ENTITY,
            AppError::PayloadTooLarge { .. }=> StatusCode::PAYLOAD_TOO_LARGE,
            AppError::NotFound { .. }       => StatusCode::NOT_FOUND,
            AppError::RangeTooLarge { .. }  => StatusCode::BAD_REQUEST,
            AppError::Conflict { .. }       => StatusCode::CONFLICT,
            AppError::Unauthorized { .. }   => StatusCode::UNAUTHORIZED,
            AppError::PasswordChangeRequired=> StatusCode::FORBIDDEN,
            _                               => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        let status = self.status_code();
        let body = self.to_json_error();
        HttpResponse::build(status)
            .content_type("application/json")
            .body(serde_json::to_string(&body).unwrap_or_default())
    }
}

impl AppError {
    /// Build the JSON error body matching the design doc format:
    /// { "error": "...", "code": "...", "details": {...} }
    pub fn to_json_error(&self) -> ErrorBody;

    /// Machine-readable error code (e.g., "NOT_FOUND", "INVALID_CSAF")
    pub fn code(&self) -> &str;
}

#[derive(serde::Serialize)]
pub struct ErrorBody {
    pub error: String,
    pub code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

/// Convenience type alias used everywhere
pub type Result<T> = std::result::Result<T, AppError>;
```

## Error Code Mapping

| Variant | HTTP Status | `code` string |
|---------|-------------|---------------|
| `Validation` | 400 | `VALIDATION_ERROR` |
| `CsafValidation` | 422 | `INVALID_CSAF` |
| `PayloadTooLarge` | 413 | `PAYLOAD_TOO_LARGE` |
| `NotFound` | 404 | `NOT_FOUND` |
| `RangeTooLarge` | 400 | `RANGE_TOO_LARGE` |
| `Conflict` | 409 | `INGEST_CONFLICT` |
| `Unauthorized` | 401 | `UNAUTHORIZED` |
| `PasswordChangeRequired` | 403 | `PASSWORD_CHANGE_REQUIRED` |
| `Database` | 500 | `INTERNAL_ERROR` |
| `Download` | 502 | `DOWNLOAD_ERROR` |
| `Parse` | 400 | `INVALID_JSON` |
| `Conversion` | 500 | `CONVERSION_ERROR` |
| `Io` | 500 | `INTERNAL_ERROR` |
| `Config` | 500 | `CONFIG_ERROR` |
| `Internal` | 500 | `INTERNAL_ERROR` |
| `External` | 500 | `INTERNAL_ERROR` |

## Implementation Steps

1. Create `src/error.rs` with the `AppError` enum
2. Implement `Display` for human-readable messages
3. Implement `Error::source()` for error chaining
4. Implement `From` for each external error type
5. Implement `ResponseError` for actix-web integration
6. Implement `to_json_error()` and `code()` helper methods
7. Define `ErrorBody` struct for JSON responses
8. Define `pub type Result<T>` alias
9. Write tests: verify each variant serializes to the correct JSON error body

## Acceptance Criteria

- [ ] `AppError::NotFound { ... }.status_code()` returns 404
- [ ] `AppError::CsafValidation { ... }.status_code()` returns 422
- [ ] `serde_json::Error` automatically converts to `AppError::Parse` via `?`
- [ ] `sqlx::Error` automatically converts to `AppError::Database` via `?`
- [ ] Error response body matches the format: `{"error":"...","code":"...","details":{...}}`
- [ ] All error responses have `Content-Type: application/json`
