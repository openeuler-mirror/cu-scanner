# 04 - Domain Models

## Purpose

Define all data structures: CSAF JSON input models (for deserialization), OVAL XML output models (for serialization), and database row models (for sqlx queries). This is the shared type system for the entire application.

## Dependencies

- 01-project-setup (for serde, quick-xml, sqlx, chrono dependencies)

## Files to Create

```
src/models/mod.rs
src/models/csaf.rs       # CSAF JSON input models
src/models/oval.rs       # OVAL XML output models
src/models/db.rs         # Database row models (sqlx FromRow)
```

## Reference Design Doc Sections

- Section 5.1 (CSAF JSON structure)
- Section 5.2 (OVAL XML structure)
- Section 5.3 (Database table structures)
- Section 6.3 (XML serialization design)

---

## 4a. CSAF Models (`src/models/csaf.rs`)

```rust
use serde::Deserialize;

/// Top-level CSAF document (matches the JSON structure in Section 5.1)
#[derive(Debug, Clone, Deserialize)]
pub struct CsafDocument {
    pub document: CsafDocumentMeta,
    pub product_tree: CsafProductTree,
    pub vulnerabilities: Vec<CsafVulnerability>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CsafDocumentMeta {
    pub title: String,
    pub csaf_version: String,
    pub category: String,
    pub publisher: CsafPublisher,
    pub tracking: CsafTracking,
    pub aggregate_severity: Option<CsafSeverity>,
    pub notes: Vec<CsafNote>,
    pub references: Vec<CsafReference>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CsafPublisher {
    pub name: String,
    pub category: String,
    pub namespace: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CsafTracking {
    pub id: String,                          // e.g. "CUOS-SA-2025-1665"
    pub version: String,                     // e.g. "1.0.0"
    pub status: String,
    pub initial_release_date: String,        // ISO 8601
    pub current_release_date: String,        // ISO 8601
}

#[derive(Debug, Clone, Deserialize)]
pub struct CsafSeverity {
    pub text: String,
    pub namespace: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CsafNote {
    pub text: String,
    pub category: String,                   // "summary", "description", "general", etc.
}

#[derive(Debug, Clone, Deserialize)]
pub struct CsafReference {
    pub summary: Option<String>,
    pub url: String,
    pub category: String,                   // "self", "external"
}

#[derive(Debug, Clone, Deserialize)]
pub struct CsafProductTree {
    pub branches: Vec<CsafBranch>,
    pub relationships: Vec<CsafRelationship>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CsafBranch {
    pub name: String,
    pub category: String,                   // "vendor", "product_name", "product_version"
    #[serde(default)]
    pub branches: Vec<CsafBranch>,
    pub product: Option<CsafProduct>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CsafProduct {
    pub name: String,
    pub product_id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CsafRelationship {
    pub category: String,                   // "default_component_of"
    pub full_product_name: CsafFullProductName,
    pub product_reference: String,          // e.g. "openssh-9.6p1-6.ule4.aarch64.rpm"
    pub relates_to_product_reference: String, // e.g. "CUOS-4.0"
}

#[derive(Debug, Clone, Deserialize)]
pub struct CsafFullProductName {
    pub name: String,
    pub product_id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CsafVulnerability {
    pub cve: Option<String>,
    pub notes: Vec<CsafNote>,
    pub product_status: CsafProductStatus,
    pub remediations: Vec<CsafRemediation>,
    pub scores: Vec<CsafCvssScore>,
    pub threats: Vec<CsafThreat>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CsafProductStatus {
    #[serde(default)]
    pub fixed: Vec<String>,
    #[serde(default)]
    pub known_affected: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CsafRemediation {
    pub category: String,                   // "vendor_fix"
    pub details: String,
    pub product_ids: Vec<String>,
    pub url: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CsafCvssScore {
    pub cvss_v3: Option<CsafCvssV3>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CsafCvssV3 {
    pub version: String,
    pub vectorString: String,
    pub baseScore: f64,
    pub baseSeverity: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CsafThreat {
    pub category: String,
    pub details: String,
}

// --- Helper types (not from JSON, derived during parsing) ---

/// Parsed RPM package info from a filename like "openssh-9.6p1-6.ule4.aarch64.rpm"
#[derive(Debug, Clone)]
pub struct RpmPackageInfo {
    pub name: String,        // "openssh" or "openssh-askpass"
    pub version: String,     // "9.6p1"
    pub release: String,     // "6.ule4"
    pub arch: String,        // "aarch64"
    pub evr: String,         // "{epoch}:{version}-{release}"
    pub epoch: String,       // "0"
    pub full_filename: String,
}

/// Extracted platform info from product_tree branches
#[derive(Debug, Clone)]
pub struct PlatformInfo {
    pub vendor: String,                      // "ChinaUnicom"
    pub product_name: String,                // "CUOS"
    pub version: String,                     // "4.0"
    pub cpe: String,                         // "cpe:/o:chinaunicom:cuos:4.0"
}

/// Summary extracted from a CSAF document for use before full conversion
#[derive(Debug, Clone)]
pub struct CsafSummary {
    pub csaf_id: String,                     // "CUOS-SA-2025-1665"
    pub oval_numeric_id: String,             // "20251665"
    pub title: String,
    pub severity: String,
    pub release_date: chrono::NaiveDate,
    pub packages: Vec<RpmPackageInfo>,
    pub cves: Vec<String>,
    pub platforms: Vec<PlatformInfo>,
}
```

---

## 4b. OVAL Models (`src/models/oval.rs`)

These models are used for **serialization** to OVAL XML. Follow the quick-xml serde patterns from design doc Section 6.3.

```rust
use serde::Serialize;

// === Root element ===

#[derive(Debug, Clone, Serialize)]
#[serde(rename = "oval_definitions")]
pub struct OvalDefinitions {
    // Namespace declarations (order matters: put xmlns first!)
    #[serde(rename = "@xmlns")]
    pub xmlns: String,
    #[serde(rename = "@xmlns:oval")]
    pub xmlns_oval: String,
    #[serde(rename = "@xmlns:unix-def")]
    pub xmlns_unix_def: String,
    #[serde(rename = "@xmlns:red-def")]
    pub xmlns_red_def: String,
    #[serde(rename = "@xmlns:ind-def")]
    pub xmlns_ind_def: String,
    #[serde(rename = "@xmlns:xsi")]
    pub xmlns_xsi: String,
    #[serde(rename = "@xsi:schemaLocation")]
    pub schema_location: String,

    pub generator: OvalGenerator,
    pub definitions: OvalDefinitionsContainer,
    pub tests: OvalTestsContainer,
    pub objects: OvalObjectsContainer,
    pub states: OvalStatesContainer,
}

// === Generator ===

#[derive(Debug, Clone, Serialize)]
pub struct OvalGenerator {
    #[serde(rename = "oval:product_name")]
    pub product_name: String,
    #[serde(rename = "oval:product_version")]
    pub product_version: String,
    #[serde(rename = "oval:schema_version")]
    pub schema_version: String,
    #[serde(rename = "oval:timestamp")]
    pub timestamp: String,                   // ISO 8601 UTC
    #[serde(rename = "oval:content_version")]
    pub content_version: String,             // Unix timestamp as string
}

// === Definitions container ===

#[derive(Debug, Clone, Serialize)]
pub struct OvalDefinitionsContainer {
    #[serde(rename = "$value")]
    pub definitions: Vec<OvalDefinition>,
}

// === Single Definition ===

#[derive(Debug, Clone, Serialize)]
pub struct OvalDefinition {
    #[serde(rename = "@id")]
    pub id: String,                          // oval:com.chinaunicom.cuos:def:20251665
    #[serde(rename = "@class")]
    pub class: String,                       // "patch"
    #[serde(rename = "@version")]
    pub version: String,                     // "1"
    pub metadata: OvalDefinitionMetadata,
    pub criteria: OvalCriteria,
}

#[derive(Debug, Clone, Serialize)]
pub struct OvalDefinitionMetadata {
    pub title: String,
    pub affected: OvalAffected,
    #[serde(rename = "reference")]
    #[serde(default)]
    pub references: Vec<OvalReference>,
    pub description: String,
    pub advisory: OvalAdvisory,
}

#[derive(Debug, Clone, Serialize)]
pub struct OvalAffected {
    #[serde(rename = "@family")]
    pub family: String,                      // "unix"
    pub platform: String,                    // "CUOS 4.0"
}

#[derive(Debug, Clone, Serialize)]
pub struct OvalReference {
    #[serde(rename = "@ref_id")]
    pub ref_id: String,
    #[serde(rename = "@ref_url")]
    pub ref_url: String,
    #[serde(rename = "@source")]
    pub source: String,                      // "CSAF" | "CVE"
}

#[derive(Debug, Clone, Serialize)]
pub struct OvalAdvisory {
    #[serde(rename = "@from")]
    pub from: String,
    pub severity: String,
    pub rights: String,
    pub issued: OvalDateElement,
    pub updated: OvalDateElement,
    #[serde(rename = "cve")]
    #[serde(default)]
    pub cves: Vec<OvalCveEntry>,
    pub affected_cpe_list: OvalCpeList,
}

#[derive(Debug, Clone, Serialize)]
pub struct OvalDateElement {
    #[serde(rename = "@date")]
    pub date: String,                        // "2025-11-10"
}

#[derive(Debug, Clone, Serialize)]
pub struct OvalCveEntry {
    #[serde(rename = "@cvss3")]
    pub cvss3: Option<String>,
    #[serde(rename = "@href")]
    pub href: Option<String>,
    #[serde(rename = "$text")]
    pub cve_id: String,                      // "CVE-2025-32728"
}

#[derive(Debug, Clone, Serialize)]
pub struct OvalCpeList {
    #[serde(rename = "cpe")]
    pub cpes: Vec<String>,                   // "cpe:/o:chinaunicom:cuos:4.0"
}

// === Criteria tree ===

#[derive(Debug, Clone, Serialize)]
pub struct OvalCriteria {
    #[serde(rename = "@operator")]
    pub operator: String,                    // "AND" | "OR"
    #[serde(rename = "$value")]
    pub children: Vec<OvalCriteriaNode>,
}

/// Each child of a criteria is either a criterion (leaf, has test_ref)
/// or a nested criteria (branch, has operator).
/// Represented as an enum with external tagging for quick-xml compatibility.
#[derive(Debug, Clone, Serialize)]
pub enum OvalCriteriaNode {
    #[serde(rename = "criterion")]
    Criterion {
        #[serde(rename = "@comment")]
        comment: String,
        #[serde(rename = "@test_ref")]
        test_ref: String,
    },
    #[serde(rename = "criteria")]
    NestedCriteria {
        #[serde(rename = "@operator")]
        operator: String,                    // "AND" | "OR"
        #[serde(rename = "$value")]
        children: Vec<OvalCriteriaNode>,
    },
}

// === Tests container (heterogeneous list) ===

#[derive(Debug, Clone, Serialize)]
pub struct OvalTestsContainer {
    #[serde(rename = "$value")]
    pub tests: Vec<OvalTest>,
}

/// Enum for heterogeneous test types in OVAL.
/// Each variant serializes to a different XML tag name (external tag enum).
#[derive(Debug, Clone, Serialize)]
pub enum OvalTest {
    #[serde(rename = "red-def:rpminfo_test")]
    RpmInfo {
        #[serde(rename = "@id")]
        id: String,
        #[serde(rename = "@version")]
        version: String,
        #[serde(rename = "@check")]
        check: String,                       // "at least one"
        #[serde(rename = "@comment")]
        comment: String,
        #[serde(rename = "red-def:object")]
        object_ref: OvalObjectRef,
        #[serde(rename = "red-def:state")]
        state_ref: OvalStateRef,
    },
    #[serde(rename = "red-def:rpmverifyfile_test")]
    RpmVerifyFile {
        #[serde(rename = "@id")]
        id: String,
        #[serde(rename = "@version")]
        version: String,
        #[serde(rename = "@check")]
        check: String,
        #[serde(rename = "@comment")]
        comment: String,
        #[serde(rename = "red-def:object")]
        object_ref: OvalObjectRef,
        #[serde(rename = "red-def:state")]
        state_ref: OvalStateRef,
    },
}

#[derive(Debug, Clone, Serialize)]
pub struct OvalObjectRef {
    #[serde(rename = "@object_ref")]
    pub object_ref: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct OvalStateRef {
    #[serde(rename = "@state_ref")]
    pub state_ref: String,
}

// === Objects container (heterogeneous list) ===

#[derive(Debug, Clone, Serialize)]
pub struct OvalObjectsContainer {
    #[serde(rename = "$value")]
    pub objects: Vec<OvalObject>,
}

#[derive(Debug, Clone, Serialize)]
pub enum OvalObject {
    #[serde(rename = "red-def:rpminfo_object")]
    RpmInfo {
        #[serde(rename = "@id")]
        id: String,
        #[serde(rename = "@version")]
        version: String,
        #[serde(rename = "red-def:name")]
        name: String,
    },
    #[serde(rename = "red-def:rpmverifyfile_object")]
    RpmVerifyFile {
        #[serde(rename = "@id")]
        id: String,
        #[serde(rename = "@version")]
        version: String,
        #[serde(rename = "red-def:behaviors")]
        behaviors: OvalRpmVerifyBehaviors,
        #[serde(rename = "red-def:name")]
        name: OvalNameWithOperation,
        #[serde(rename = "red-def:filepath")]
        filepath: String,                    // "/etc/os-release"
    },
}

#[derive(Debug, Clone, Serialize)]
pub struct OvalRpmVerifyBehaviors {
    #[serde(rename = "@noconfigfiles")]
    pub noconfigfiles: String,               // "true"
    #[serde(rename = "@noghostfiles")]
    pub noghostfiles: String,                // "true"
    #[serde(rename = "@nogroup")]
    pub nogroup: String,                     // "true"
    #[serde(rename = "@nolinkto")]
    pub nolinkto: String,                    // "true"
    #[serde(rename = "@nomd5")]
    pub nomd5: String,                       // "true"
    #[serde(rename = "@nomode")]
    pub nomode: String,                      // "true"
    #[serde(rename = "@nomtime")]
    pub nomtime: String,                     // "true"
    #[serde(rename = "@nordev")]
    pub nordev: String,                      // "true"
    #[serde(rename = "@nosize")]
    pub nosize: String,                      // "true"
    #[serde(rename = "@nouser")]
    pub nouser: String,                      // "true"
}

#[derive(Debug, Clone, Serialize)]
pub struct OvalNameWithOperation {
    #[serde(rename = "@operation")]
    pub operation: String,                   // "pattern match"
    #[serde(rename = "$text")]
    pub value: String,                       // "^os-release$"
}

// === States container (heterogeneous list) ===

#[derive(Debug, Clone, Serialize)]
pub struct OvalStatesContainer {
    #[serde(rename = "$value")]
    pub states: Vec<OvalState>,
}

#[derive(Debug, Clone, Serialize)]
pub enum OvalState {
    #[serde(rename = "red-def:rpminfo_state")]
    RpmInfo {
        #[serde(rename = "@id")]
        id: String,
        #[serde(rename = "@version")]
        version: String,
        // Optional child elements (only include if value is Some)
        #[serde(rename = "red-def:evr", skip_serializing_if = "Option::is_none")]
        evr: Option<OvalEvrElement>,
        #[serde(rename = "red-def:signature_keyid", skip_serializing_if = "Option::is_none")]
        signature_keyid: Option<OvalSignatureKeyidElement>,
    },
    #[serde(rename = "red-def:rpmverifyfile_state")]
    RpmVerifyFile {
        #[serde(rename = "@id")]
        id: String,
        #[serde(rename = "@version")]
        version: String,
        #[serde(rename = "red-def:name")]
        name: OvalNameWithOperation,
        #[serde(rename = "red-def:version")]
        version_pattern: OvalNameWithOperation,
    },
}

#[derive(Debug, Clone, Serialize)]
pub struct OvalEvrElement {
    #[serde(rename = "@datatype")]
    pub datatype: String,                    // "evr_string"
    #[serde(rename = "@operation")]
    pub operation: String,                   // "less than"
    #[serde(rename = "$text")]
    pub value: String,                       // "0:9.6p1-6.ule4"
}

#[derive(Debug, Clone, Serialize)]
pub struct OvalSignatureKeyidElement {
    #[serde(rename = "@operation")]
    pub operation: String,                   // "equals"
    #[serde(rename = "$text")]
    pub value: String,                       // signature key id
}
```

---

## 4c. Database Row Models (`src/models/db.rs`)

Each struct maps to a table row. Use `sqlx::FromRow` derive.

```rust
use chrono::{DateTime, Utc, NaiveDate, NaiveDateTime};
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow)]
pub struct CsafSource {
    pub id: i64,
    pub csaf_id: String,
    pub file_name: String,
    pub title: Option<String>,
    pub category: Option<String>,
    pub severity: Option<String>,
    pub release_date: Option<NaiveDateTime>,
    pub csaf_version: Option<String>,
    pub tracking_version: Option<String>,
    pub download_url: Option<String>,
    pub downloaded_at: Option<NaiveDateTime>,
    pub parsed_at: Option<NaiveDateTime>,
    pub oval_numeric_id: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, FromRow)]
pub struct OvalDefinitionRow {
    pub id: i64,
    pub csaf_id: String,
    pub oval_id: String,
    pub class: String,
    pub version: i32,
    pub title: Option<String>,
    pub description: Option<String>,
    pub family: Option<String>,
    pub platform: Option<String>,
    pub severity: Option<String>,
    pub issued_date: Option<NaiveDate>,
    pub updated_date: Option<NaiveDate>,
    pub rights: Option<String>,
    pub advisory_from: Option<String>,
    pub generator_timestamp: Option<NaiveDateTime>,
    pub content_version: Option<i64>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, FromRow)]
pub struct OvalReferenceRow {
    pub id: i64,
    pub definition_id: i64,
    pub ref_id: Option<String>,
    pub ref_url: Option<String>,
    pub source: Option<String>,
}

#[derive(Debug, Clone, FromRow)]
pub struct OvalCveRow {
    pub id: i64,
    pub definition_id: i64,
    pub cve_id: Option<String>,
    pub cvss3: Option<String>,
    pub impact: Option<String>,
    pub href: Option<String>,
    pub public_date: Option<NaiveDate>,
}

#[derive(Debug, Clone, FromRow)]
pub struct OvalCpeRow {
    pub id: i64,
    pub definition_id: i64,
    pub cpe: Option<String>,
}

#[derive(Debug, Clone, FromRow)]
pub struct OvalTestRow {
    pub id: i64,
    pub oval_id: String,
    pub definition_id: i64,
    pub test_type: Option<String>,
    pub check: Option<String>,
    pub comment: Option<String>,
    pub version: Option<i32>,
    pub object_ref: Option<String>,
    pub state_ref: Option<String>,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, FromRow)]
pub struct OvalObjectRow {
    pub id: i64,
    pub oval_id: String,
    pub object_type: Option<String>,
    pub name: Option<String>,
    pub rpm_version: Option<String>,
    pub filepath: Option<String>,
    pub version: Option<i32>,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, FromRow)]
pub struct OvalStateRow {
    pub id: i64,
    pub oval_id: String,
    pub state_type: Option<String>,
    pub evr: Option<String>,
    pub evr_operation: Option<String>,
    pub signature_keyid: Option<String>,
    pub name_pattern: Option<String>,
    pub version_pattern: Option<String>,
    pub version: Option<i32>,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, FromRow)]
pub struct OvalCriteriaRow {
    pub id: i64,
    pub definition_id: i64,
    pub parent_id: Option<i64>,
    pub operator: Option<String>,
    pub criterion_test_ref: Option<String>,
    pub criterion_comment: Option<String>,
    pub sequence: Option<i32>,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, FromRow)]
pub struct DownloadTaskRow {
    pub id: i64,
    pub file_name: String,
    pub source_url: Option<String>,
    pub status: Option<String>,
    pub attempt_count: Option<i32>,
    pub error_message: Option<String>,
    pub sync_batch_id: Option<String>,
    pub created_at: NaiveDateTime,
    pub completed_at: Option<NaiveDateTime>,
}

#[derive(Debug, Clone, FromRow)]
pub struct UserRow {
    pub id: i64,
    pub username: String,
    pub password_hash: String,
    pub role: String,
    pub status: Option<String>,
    pub failed_attempts: Option<i32>,
    pub locked_until: Option<NaiveDateTime>,
    pub last_login_at: Option<NaiveDateTime>,
    pub must_change_password: Option<bool>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, FromRow)]
pub struct EpochCacheRow {
    pub id: i64,
    pub pkg_name: String,
    pub repo_id: String,
    pub epoch: String,
    pub source: Option<String>,
    pub resolved_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
}
```

---

## `src/models/mod.rs`

```rust
pub mod csaf;
pub mod oval;
pub mod db;

// Re-export commonly used types
pub use csaf::CsafDocument;
pub use oval::OvalDefinitions;
```

## Implementation Steps

1. Create `src/models/mod.rs` with module declarations and re-exports
2. Create `src/models/csaf.rs`:
   - Define all CSAF structs with `#[derive(Deserialize)]`
   - Implement helper methods on `CsafDocument`:
     - `fn csaf_id(&self) -> &str` — returns `tracking.id`
     - `fn oval_numeric_id(&self) -> String` — extracts `YYYYNNNN` from `CUOS-SA-YYYY-NNNN`
     - `fn extract_packages(&self) -> Vec<RpmPackageInfo>` — parses RPM filenames
     - `fn extract_platforms(&self) -> Vec<PlatformInfo>` — extracts vendor/product/version
     - `fn extract_cves(&self) -> Vec<String>` — extracts CVE IDs
     - `fn summary(&self) -> CsafSummary` — builds the summary struct
3. Create `src/models/oval.rs`:
   - Define all OVAL structs with `#[derive(Serialize)]` and quick-xml annotations
   - Implement `OvalDefinitions::new(config: &OvalConfig)` — creates empty root with xmlns, generator
4. Create `src/models/db.rs`:
   - Define all row structs with `#[derive(FromRow)]`
5. Write tests:
   - Deserialize a sample CSAF JSON and verify field extraction
   - Serialize a constructed OVAL document and verify XML output (compare against expected XML snippet)
   - Parse an RPM filename and verify name/version/release/arch extraction

## RPM Filename Parsing Algorithm

Given `"openssh-askpass-9.6p1-6.ule4.aarch64.rpm"`:
1. Strip `.rpm` suffix → `"openssh-askpass-9.6p1-6.ule4.aarch64"`
2. Find the arch: last `-` segment → `"aarch64"`
3. Find the release: second-to-last `-` segment → `"6.ule4"`
4. Find the version: third-to-last `-` segment → `"9.6p1"`
5. Everything before the version is the name → `"openssh-askpass"`

Since RPM names can contain hyphens, parse from right to left.

## Acceptance Criteria

- [ ] Sample CSAF JSON deserializes without error into `CsafDocument`
- [ ] `csaf_id()` returns `"CUOS-SA-2025-1665"`
- [ ] `oval_numeric_id()` returns `"20251665"` from `"CUOS-SA-2025-1665"`
- [ ] RPM filename `"openssh-askpass-9.6p1-6.ule4.aarch64.rpm"` correctly parses name=`"openssh-askpass"`, version=`"9.6p1"`, release=`"6.ule4"`, arch=`"aarch64"`
- [ ] RPM filename `"openssh-9.6p1-6.ule4.aarch64.rpm"` correctly parses name=`"openssh"`, version=`"9.6p1"`, release=`"6.ule4"`, arch=`"aarch64"`
- [ ] Serialized OVAL XML has correct namespace declarations (6 xmlns attributes on root)
- [ ] `OvalCriteriaNode` enum correctly serializes to `<criterion>` and nested `<criteria>` elements
- [ ] All DB row structs compile with `#[derive(FromRow)]`
