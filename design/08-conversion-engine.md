# 08 - Conversion Engine

## Purpose

Convert a parsed CSAF document into an OVAL document structure. Includes OVAL ID generation, platform detection, signature verification logic, and RPM version detection.

## Dependencies

- 02-config (OvalConfig)
- 03-error-types
- 04-domain-models (CsafDocument, OVAL models)
- 06-csaf-parser (extraction helpers)
- 07-epoch-resolver

## Files to Create

```
src/engine/mod.rs
src/engine/id_gen.rs
src/engine/converter.rs
```

## Reference Design Doc Sections

- Section 6.3 (Conversion Engine, ID Generation, Platform Detection, Signature Verification)
- Section 5.2 (OVAL Output Structure)

---

## 8a. ID Generator (`src/engine/id_gen.rs`)

```rust
/// Generates OVAL IDs for a single CSAF advisory.
/// Sequence numbering: 0001-0100 platform, 0101-0200 signature, 0201-9999 version.
pub struct IdGenerator {
    numeric_id: String,            // e.g. "20251665"
    next_signature_seq: u32,       // starts at 101
    next_version_seq: u32,         // starts at 201
}

impl IdGenerator {
    pub fn new(numeric_id: &str) -> Self { /* ... */ }

    /// Generate Definition ID
    /// "oval:com.chinaunicom.cuos:def:20251665"
    pub fn def_id(&self) -> String;

    /// Generate platform detection IDs for a given platform sequence number.
    /// Platform seq mapping: CUOS 4.0→1, CUOS 5.0→2, openEuler 20.03→20, etc.
    /// Returns (test_id, object_id, state_id)
    pub fn platform_ids(&self, platform_seq: u32) -> (String, String, String);

    /// Generate next signature verification IDs.
    /// Returns (test_id, object_id, state_id)
    pub fn next_signature_ids(&mut self) -> (String, String, String);

    /// Generate next version detection IDs.
    /// Returns (test_id, object_id, state_id)
    pub fn next_version_ids(&mut self) -> (String, String, String);

    /// Internal: format a sequence number into test/object/state IDs
    fn format_ids(&self, seq: u32) -> (String, String, String) {
        let seq_str = format!("{:04}", seq);
        let base = &self.numeric_id;
        (
            format!("oval:com.chinaunicom.cuos:tst:{}{}", base, seq_str),
            format!("oval:com.chinaunicom.cuos:obj:{}{}", base, seq_str),
            format!("oval:com.chinaunicom.cuos:ste:{}{}", base, seq_str),
        )
    }
}

/// Map (platform_name, platform_version) → sequence number.
/// CUOS 4.0 → 1, CUOS 5.0 → 2, openEuler 20.03 → 20, openEuler 22.03 → 22, openEuler 24.03 → 24
pub fn platform_seq_number(distribution: &str, version: &str) -> u32;
```

---

## 8b. Converter (`src/engine/converter.rs`)

```rust
use crate::config::OvalConfig;
use crate::error::Result;
use crate::models::csaf::CsafDocument;
use crate::models::csaf::RpmPackageInfo;
use crate::models::oval::*;
use crate::epoch::YumEpochResolver;

/// Converts a CSAF document into an OVAL document.
pub struct Converter {
    config: OvalConfig,
    epoch_resolver: YumEpochResolver,
}

impl Converter {
    pub fn new(config: OvalConfig, epoch_resolver: YumEpochResolver) -> Self;

    /// Main conversion entry point.
    /// 1. Parse CSAF → validate
    /// 2. Extract packages, platforms, CVEs
    /// 3. Resolve epochs
    /// 4. Build OVAL definitions, tests, objects, states, criteria
    pub async fn convert(&mut self, csaf: &CsafDocument) -> Result<OvalDefinitions>;

    /// Build the generator section
    fn build_generator(&self) -> OvalGenerator;

    /// Build the single definition for this CSAF
    async fn build_definition(&mut self, csaf: &CsafDocument, gen: &mut IdGenerator) -> Result<(OvalDefinition, Vec<OvalTest>, Vec<OvalObject>, Vec<OvalState>, OvalCriteria)>;

    /// Build platform detection test/object/state
    fn build_platform_detection(&self, gen: &mut IdGenerator, platform: &PlatformInfo) -> (OvalTest, OvalObject, OvalState);

    /// Build signature verification test/object/state (only if config.signature_keyid is set AND mode is server)
    fn build_signature_check(&self, gen: &mut IdGenerator, pkg: &RpmPackageInfo) -> Option<(OvalTest, OvalObject, OvalState)>;

    /// Build version detection test/object/state for a package
    fn build_version_check(&self, gen: &mut IdGenerator, pkg: &RpmPackageInfo) -> (OvalTest, OvalObject, OvalState);

    /// Build the criteria tree:
    /// AND(
    ///   criterion(platform detection),
    ///   OR(
    ///     AND(criterion(version_check_pkg1), criterion(signature_check_pkg1)),
    ///     AND(criterion(version_check_pkg2), criterion(signature_check_pkg2)),
    ///     ...
    ///   )
    /// )
    fn build_criteria(&self, platform_test_ref: &str, version_refs: &[(String, Option<String>)]) -> OvalCriteria;
}
```

---

## Key Algorithms

### 1. Platform Detection

Map `PlatformInfo` to OVAL components:

```rust
fn build_platform_detection(&self, gen: &mut IdGenerator, platform: &PlatformInfo) -> (OvalTest, OvalObject, OvalState) {
    let (tst_id, obj_id, ste_id) = gen.platform_ids(platform_seq_number(&platform.distribution, &platform.version));

    // Determine version pattern for rpmverifyfile_state
    let version_pattern = match platform.distribution.to_lowercase().as_str() {
        "cuos" => "^CUOS".to_string(),
        "openeuler" => "^openEuler|^.*ID=\"openEuler\"".to_string(),
        _ => format!("^{}", platform.distribution),
    };

    // Build test
    let test = OvalTest::RpmVerifyFile {
        id: tst_id.clone(),
        version: "1".into(),
        check: "at least one".into(),
        comment: format!("{} {} is installed", platform.product_name, platform.version),
        object_ref: OvalObjectRef { object_ref: obj_id.clone() },
        state_ref: OvalStateRef { state_ref: ste_id.clone() },
    };

    // Build object
    let object = OvalObject::RpmVerifyFile {
        id: obj_id,
        version: "1".into(),
        behaviors: OvalRpmVerifyBehaviors { /* all "true" */ },
        name: OvalNameWithOperation { operation: "pattern match".into(), value: "^os-release$".into() },
        filepath: "/etc/os-release".into(),
    };

    // Build state
    let state = OvalState::RpmVerifyFile {
        id: ste_id,
        version: "1".into(),
        name: OvalNameWithOperation { operation: "pattern match".into(), value: "^os-release$".into() },
        version_pattern: OvalNameWithOperation { operation: "pattern match".into(), value: version_pattern },
    };

    (test, object, state)
}
```

### 2. Version Detection

```rust
fn build_version_check(&self, gen: &mut IdGenerator, pkg: &RpmPackageInfo) -> (OvalTest, OvalObject, OvalState) {
    let (tst_id, obj_id, ste_id) = gen.next_version_ids();

    let test = OvalTest::RpmInfo {
        id: tst_id.clone(),
        version: "1".into(),
        check: "at least one".into(),
        comment: format!("{} is earlier than {}", pkg.name, pkg.evr),
        object_ref: OvalObjectRef { object_ref: obj_id.clone() },
        state_ref: OvalStateRef { state_ref: ste_id.clone() },
    };

    let object = OvalObject::RpmInfo {
        id: obj_id,
        version: "1".into(),
        name: pkg.name.clone(),
    };

    let state = OvalState::RpmInfo {
        id: ste_id,
        version: "1".into(),
        evr: Some(OvalEvrElement {
            datatype: "evr_string".into(),
            operation: "less than".into(),
            value: pkg.evr.clone(),
        }),
        signature_keyid: None,
    };

    (test, object, state)
}
```

### 3. Signature Verification (optional)

Only generated when `config.enable_signature_check = true` and `config.signature_keyid` is Some. Uses `gen.next_signature_ids()`.

### 4. Criteria Tree Construction

```
AND
├── criterion: "{platform} is installed" → platform_test_ref
└── OR
    ├── AND
    │   ├── criterion: "openssh is earlier than 0:9.6p1-6.ule4" → version_test_ref[0]
    │   └── criterion: "openssh is signed with CUOS key" → sig_test_ref[0]
    ├── AND
    │   ├── criterion: "openssh-clients is earlier than ..." → version_test_ref[1]
    │   └── criterion: "openssh-clients is signed with CUOS key" → sig_test_ref[1]
    └── ...
```

### 5. Field Mapping (CSAF → OVAL)

| CSAF Field | OVAL Field | Transform |
|-----------|------------|-----------|
| `tracking.id` | `definition.id` | `CUOS-SA-YYYY-NNNN` → `oval:com.chinaunicom.cuos:def:YYYYNNNN` |
| `title` + `severity` | `metadata.title` | `"{csaf_id}: {title} ({severity})"` |
| `tracking.current_release_date` | `advisory.issued/updated` | Extract date part `YYYY-MM-DD` |
| `aggregate_severity.text` | `advisory.severity` | Direct copy |
| `notes[category=summary]` | `metadata.description` | First matching note text |
| `references[category=self]` | `reference[source=CSAF]` | `ref_id` = tracking.id, `ref_url` = ref.url |
| `references[category=external]` | `reference[source=CVE]` | Extract CVE from summary or URL |
| `product_tree` | `affected/platform` + `advisory.affected_cpe_list` | Build CPE strings |
| `vulnerabilities[].cve` | `advisory.cve` | CVE ID + CVSS vector |
| `vulnerabilities[].product_status.fixed` | `tests/objects/states` | Generate RPM version detection |

## CLI vs Server Mode Behavior

- **CLI mode** (`convert` subcommand): Does NOT generate signature verification criterion, even if config has `signature_keyid`
- **Server mode** (`server` / `sync`): Respects `signature_keyid` config

The converter receives a `mode: ConversionMode` parameter:

```rust
pub enum ConversionMode {
    Cli,      // No signature checks
    Server,   // Respect config.enable_signature_check
}
```

## Test Cases

1. **Basic conversion**: Parse sample CSAF → convert → verify definition ID, title, severity
2. **Platform detection**: Verify generated platform test references `/etc/os-release` with correct pattern
3. **Multiple packages**: CSAF with 3 packages → generates 3 version check tests + 1 platform test (total 4)
4. **Signature check enabled**: Server mode + signature_keyid set → signature tests generated
5. **Signature check disabled**: CLI mode → no signature tests
6. **ID generation**: 3 packages → seq 0201, 0202, 0203 for version checks
7. **Criteria tree structure**: Verify AND(platform_criterion, OR(pkg1_AND, pkg2_AND, ...))
8. **Epoch in EVR**: Packages use resolved epoch value in EVR string

## Acceptance Criteria

- [ ] Converter produces valid `OvalDefinitions` from a `CsafDocument`
- [ ] Definition ID matches `oval:com.chinaunicom.cuos:def:{numeric_id}` format
- [ ] Platform detection uses `rpmverifyfile_test` with correct filepath `/etc/os-release`
- [ ] Version detection uses `rpminfo_test` with EVR less-than comparison
- [ ] Signature verification only generated in server mode with key configured
- [ ] ID sequences are correctly assigned (platform=0001-0100, sig=0101-0200, ver=0201+)
- [ ] Criteria tree has correct AND/OR nesting
- [ ] Generator section has correct product_name, schema_version, timestamp
- [ ] Epoch values from resolver are used in EVR strings
