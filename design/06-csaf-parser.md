# 06 - CSAF Parser

## Purpose

Parse CSAF JSON content into the `CsafDocument` domain model, validate required fields, and extract derived information (RPM packages, platforms, CVEs, severity).

## Dependencies

- 03-error-types
- 04-domain-models (csaf.rs)

## Files to Create

```
src/parser/mod.rs
src/parser/csaf.rs
```

## Reference Design Doc Sections

- Section 5.1 (CSAF JSON structure)
- Section 6.3 (RPM package detection logic, CSAF→OVAL field mapping)
- Section 7.2.6 (CSAF upload validation)
- Section 9.7 (Input validation)

## Public API Surface

```rust
// src/parser/csaf.rs

use crate::error::{AppError, Result};
use crate::models::csaf::*;

/// Parse CSAF JSON string into a CsafDocument.
/// Performs structural validation.
pub fn parse_csaf(json: &str) -> Result<CsafDocument>;

/// Parse CSAF from bytes (for HTTP upload).
pub fn parse_csaf_from_bytes(data: &[u8]) -> Result<CsafDocument>;

/// Validate that a CsafDocument has all required fields.
/// Returns a list of missing/invalid fields on failure.
pub fn validate_csaf(doc: &CsafDocument) -> Result<()>;

/// Extract an OvalNumericId from a CSAF tracking ID.
/// "CUOS-SA-2025-1665" → "20251665"
pub fn extract_oval_numeric_id(tracking_id: &str) -> Result<String>;

/// Parse an RPM filename into name, version, release, arch.
/// "openssh-9.6p1-6.ule4.aarch64.rpm" → RpmPackageInfo
/// Parses from right to left to handle names with hyphens.
pub fn parse_rpm_filename(filename: &str) -> Result<RpmPackageInfo>;

/// Extract all RPM packages from a CsafDocument's product_tree relationships.
pub fn extract_packages(doc: &CsafDocument) -> Vec<RpmPackageInfo>;

/// Extract platform info from product_tree branches.
pub fn extract_platforms(doc: &CsafDocument) -> Vec<PlatformInfo>;

/// Extract CVE IDs from vulnerabilities.
pub fn extract_cves(doc: &CsafDocument) -> Vec<String>;

/// Extract the severity string (Low/Moderate/Important/Critical).
pub fn extract_severity(doc: &CsafDocument) -> String;

/// Extract the description from document notes (category="summary" or "description").
pub fn extract_description(doc: &CsafDocument) -> String;

/// Extract CSAF references (self → CSAF, external → CVE).
pub fn extract_references(doc: &CsafDocument) -> Vec<CsafReferenceInfo>;

/// Extract CVSS vector strings from vulnerabilities.
pub fn extract_cvss_info(doc: &CsafDocument) -> Vec<CvssInfo>;

#[derive(Debug, Clone)]
pub struct CsafReferenceInfo {
    pub ref_id: String,
    pub ref_url: String,
    pub source: String,  // "CSAF" | "CVE"
}

#[derive(Debug, Clone)]
pub struct CvssInfo {
    pub cve_id: String,
    pub cvss3_vector: String,
    pub base_score: f64,
    pub base_severity: String,
}

/// Build a CsafSummary for quick overview without full conversion.
pub fn build_summary(doc: &CsafDocument) -> Result<CsafSummary>;
```

## Implementation Steps

### Step 1: JSON Parsing

```rust
pub fn parse_csaf(json: &str) -> Result<CsafDocument> {
    serde_json::from_str::<CsafDocument>(json)
        .map_err(|e| AppError::Parse {
            message: format!("Failed to parse CSAF JSON: {}", e),
            file: None,
        })
}
```

### Step 2: Validation

Check these required fields and return `INVALID_CSAF` with a list of missing fields:
- `document.tracking.id` — must match pattern `^[A-Z]+-[A-Z]+-\d{4}-\d{4,}$`
- `document.tracking.current_release_date` — must be valid ISO 8601
- `document.title` — must not be empty
- `product_tree` — must have branches or relationships
- `vulnerabilities` — may be empty (valid CSAF without vulns)

```rust
pub fn validate_csaf(doc: &CsafDocument) -> Result<()> {
    let mut missing = Vec::new();

    if doc.document.tracking.id.is_empty() {
        missing.push("document.tracking.id: missing".to_string());
    }
    if doc.document.title.is_empty() {
        missing.push("document.title: missing".to_string());
    }
    // ... check each required field

    if !missing.is_empty() {
        return Err(AppError::CsafValidation {
            message: "CSAF validation failed".into(),
            fields: missing,
        });
    }
    Ok(())
}
```

### Step 3: RPM Filename Parsing (Right-to-Left)

```rust
pub fn parse_rpm_filename(filename: &str) -> Result<RpmPackageInfo> {
    // 1. Strip ".rpm" suffix
    let base = filename.strip_suffix(".rpm")
        .ok_or_else(|| AppError::Parse {
            message: format!("Not an RPM file: {}", filename),
            file: Some(filename.into()),
        })?;

    // 2. Split by '-', find the arch (last segment)
    let parts: Vec<&str> = base.split('-').collect();
    if parts.len() < 4 {
        return Err(AppError::Parse {
            message: format!("Invalid RPM filename: {}", filename),
            file: Some(filename.into()),
        });
    }

    let arch = parts[parts.len() - 1].to_string();

    // 3. Find release: matches pattern like "6.ule4", "1.el8"
    let release = parts[parts.len() - 2].to_string();

    // 4. Find version: matches pattern like "9.6p1", "3.0.14"
    let version = parts[parts.len() - 3].to_string();

    // 5. Everything before the version is the name
    let name = parts[..parts.len() - 3].join("-");

    Ok(RpmPackageInfo {
        name,
        version,
        release,
        arch,
        evr: String::new(),     // filled later by epoch resolver
        epoch: "0".to_string(), // default, updated by epoch resolver
        full_filename: filename.to_string(),
    })
}
```

### Step 4: Extract Packages from product_tree

Iterate `product_tree.relationships` and find entries with `category = "default_component_of"`. Extract `product_reference` (the RPM filename) and parse each one.

### Step 5: Extract Platforms from product_tree

Walk the `branches` tree:
- Top level: `category = "vendor"` → vendor name
- Second level: `category = "product_name"` → product name
- Third level: `category = "product_version"` → version

Build CPE string: `cpe:/o:{vendor_lower}:{product_lower}:{version}`

### Step 6: Extract CVEs

Iterate `vulnerabilities`, collect all non-None `cve` fields.

### Step 7: Build Summary

Combine all extractions into a `CsafSummary`:
```rust
pub fn build_summary(doc: &CsafDocument) -> Result<CsafSummary> {
    let csaf_id = doc.document.tracking.id.clone();
    let oval_numeric_id = extract_oval_numeric_id(&csaf_id)?;
    let title = doc.document.title.clone();
    let severity = extract_severity(doc);
    let release_date = parse_date(&doc.document.tracking.current_release_date)?;
    let packages = extract_packages(doc);
    let cves = extract_cves(doc);
    let platforms = extract_platforms(doc);

    Ok(CsafSummary { csaf_id, oval_numeric_id, title, severity, release_date, packages, cves, platforms })
}
```

## Test Cases

1. **Valid CSAF parse**: Parse the sample JSON from Section 5.1, verify all fields populated
2. **RPM filename simple**: `"openssh-9.6p1-6.ule4.aarch64.rpm"` → name=`"openssh"`, version=`"9.6p1"`, release=`"6.ule4"`, arch=`"aarch64"`
3. **RPM filename with hyphens**: `"openssh-askpass-9.6p1-6.ule4.aarch64.rpm"` → name=`"openssh-askpass"`
4. **RPM filename noarch**: `"cuos-release-4.0-1.ule4.noarch.rpm"` → arch=`"noarch"`
5. **Oval numeric ID**: `"CUOS-SA-2025-1665"` → `"20251665"`
6. **Missing required field**: doc without `tracking.id` → `CsafValidation` error with field list
7. **Empty vulnerabilities**: doc with `"vulnerabilities": []` → parse succeeds, empty CVE list
8. **Platform extraction**: branches with vendor="ChinaUnicom", product="CUOS", version="4.0" → CPE `"cpe:/o:chinaunicom:cuos:4.0"`

## Acceptance Criteria

- [ ] Valid CSAF JSON parses without error
- [ ] CSAF with missing `document.tracking.id` returns `CsafValidation` with the field listed
- [ ] RPM filename parsing handles all four test cases correctly
- [ ] `extract_oval_numeric_id` works for all valid CSAF IDs
- [ ] Platform extraction handles multi-level branches
- [ ] Empty vulnerabilities array is valid
- [ ] Invalid JSON returns `AppError::Parse`
- [ ] `build_summary` returns complete summary with all fields populated
