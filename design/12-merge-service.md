# 12 - Merge Service

## Purpose

Query multiple OVAL definitions from the database by month or date range, build a merged OVAL XML document with deduplication of tests/objects/states, and return the merged XML.

## Dependencies

- 02-config (OvalConfig)
- 03-error-types
- 05-database (all oval_* repositories)
- 09-xml-serializer (serialize_to_string, OvalStreamWriter)

## Files to Create

```
src/merge/mod.rs
src/merge/service.rs
```

## Reference Design Doc Sections

- Section 5.2.2 (Merged OVAL specification)
- Section 6.3 (Deduplication strategy, Section 5)
- Section 7.2.3 (GET /oval/month/{year-month})
- Section 7.2.4 (GET /oval/range)
- Section 14.1 (Performance: max 5000 definitions per merge)

## Public API Surface

```rust
// src/merge/service.rs

use crate::error::Result;
use crate::config::OvalConfig;
use chrono::NaiveDate;

/// Maximum number of definitions allowed in a single merge to prevent memory issues
pub const MAX_MERGE_DEFINITIONS: usize = 5000;

/// Service for merging multiple OVAL definitions into one XML document.
pub struct MergeService {
    pool: AnyPool,
    config: OvalConfig,
}

impl MergeService {
    pub fn new(pool: AnyPool, config: OvalConfig) -> Self;

    /// Merge all definitions with issued_date in a given month (YYYY-MM).
    /// For each csaf_id, takes the highest version.
    /// Returns merged OVAL XML string.
    pub async fn merge_by_month(
        &self,
        year: i32,
        month: u32,
        pretty: bool,
    ) -> Result<String>;

    /// Merge all definitions with issued_date in a date range [start, end].
    /// Returns merged OVAL XML string.
    /// Validates: range ≤ 365 days, start ≤ end.
    pub async fn merge_by_range(
        &self,
        start: NaiveDate,
        end: NaiveDate,
        pretty: bool,
    ) -> Result<String>;

    /// Merge a specific set of oval_ids (for future use, e.g., batch by IDs).
    pub async fn merge_by_ids(
        &self,
        oval_ids: &[String],
        pretty: bool,
    ) -> Result<String>;
}
```

## Key Algorithms

### 1. Fetch Definitions for Month

```rust
pub async fn merge_by_month(&self, year: i32, month: u32, pretty: bool) -> Result<String> {
    let start = NaiveDate::from_ymd_opt(year, month, 1)
        .ok_or_else(|| AppError::Validation { ... })?;
    let end = if month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1).unwrap() - chrono::Duration::days(1)
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1).unwrap() - chrono::Duration::days(1)
    };

    self.merge_by_range(start, end, pretty).await
}
```

### 2. Fetch and Deduplicate

```rust
pub async fn merge_by_range(&self, start: NaiveDate, end: NaiveDate, pretty: bool) -> Result<String> {
    // Validate range
    let span = (end - start).num_days();
    if span < 0 {
        return Err(AppError::Validation { ... });
    }
    if span > 365 {
        return Err(AppError::RangeTooLarge { ... });
    }

    // 1. Query definitions in date range (latest version per csaf_id)
    let definitions = self.repo_def.find_latest_by_date_range(start, end).await?;

    if definitions.is_empty() {
        return Err(AppError::NotFound { ... });
    }

    if definitions.len() > MAX_MERGE_DEFINITIONS {
        return Err(AppError::RangeTooLarge {
            message: format!("{} definitions exceeds max {}", definitions.len(), MAX_MERGE_DEFINITIONS),
        });
    }

    let def_ids: Vec<i64> = definitions.iter().map(|d| d.id).collect();

    // 2. Fetch all related components
    let tests = self.repo_test.find_by_definition_ids(&def_ids).await?;
    let objects = self.repo_object.find_by_definition_ids(&def_ids).await?;
    let states = self.repo_state.find_by_definition_ids(&def_ids).await?;
    let criteria_list = self.repo_criteria.find_by_definition_ids(&def_ids).await?;
    let references = self.repo_ref.find_by_definition_ids(&def_ids).await?;
    let cves = self.repo_cve.find_by_definition_ids(&def_ids).await?;
    let cpes = self.repo_cpe.find_by_definition_ids(&def_ids).await?;

    // 3. Deduplicate tests/objects/states by oval_id
    let unique_tests = deduplicate_by_id(tests, |t| &t.oval_id);
    let unique_objects = deduplicate_by_id(objects, |o| &o.oval_id);
    let unique_states = deduplicate_by_id(states, |s| &s.oval_id);

    // 4. Build merged OvalDefinitions
    let oval = self.build_merged_oval(definitions, unique_tests, unique_objects, unique_states, criteria_list, references, cves, cpes)?;

    // 5. Serialize to XML
    crate::xml::serialize_to_string(&oval, pretty)
}
```

### 3. Deduplication

```rust
/// Deduplicate items by a key function. First occurrence wins.
fn deduplicate_by_id<T, F, K>(items: Vec<T>, key_fn: F) -> Vec<T>
where
    F: Fn(&T) -> &K,
    K: Eq + std::hash::Hash,
{
    let mut seen = std::collections::HashSet::new();
    items.into_iter()
        .filter(|item| seen.insert(key_fn(item)))
        .collect()
}
```

Deduplication rule: Same `oval_id` → keep first, drop rest. Content-based dedup is NOT performed (different IDs are always kept, even if content is identical).

### 4. Build Merged OVAL

```rust
fn build_merged_oval(
    &self,
    definitions: Vec<OvalDefinitionRow>,
    tests: Vec<OvalTestRow>,
    objects: Vec<OvalObjectRow>,
    states: Vec<OvalStateRow>,
    criteria_list: Vec<OvalCriteriaRow>,
    references: Vec<OvalReferenceRow>,
    cves: Vec<OvalCveRow>,
    cpes: Vec<OvalCpeRow>,
) -> Result<OvalDefinitions> {
    // Build generator with current timestamp
    let generator = self.build_generator();

    // Convert DB rows to OVAL model structs
    let oval_defs = definitions.iter().map(|d| self.row_to_definition(d, &references, &cves, &cpes, &criteria_list)).collect();
    let oval_tests = tests.iter().map(|t| self.row_to_test(t)).collect();
    let oval_objects = objects.iter().map(|o| self.row_to_object(o)).collect();
    let oval_states = states.iter().map(|s| self.row_to_state(s)).collect();

    Ok(OvalDefinitions {
        xmlns: ...,  // from DEFAULT_NAMESPACES
        generator,
        definitions: OvalDefinitionsContainer { definitions: oval_defs },
        tests: OvalTestsContainer { tests: oval_tests },
        objects: OvalObjectsContainer { objects: oval_objects },
        states: OvalStatesContainer { states: oval_states },
    })
}
```

**Key**: Converting DB rows back to OVAL model structs requires reconstruction logic (the reverse of how they were stored during ingest). The `criteria` table's adjacency list must be reconstructed into a tree.

### 5. Criteria Tree Reconstruction

Each definition's criteria rows (adjacency list model) must be rebuilt into the `OvalCriteria` tree structure:

```rust
fn build_criteria_tree(
    criteria_rows: &[OvalCriteriaRow],
    definition_id: i64,
) -> Result<OvalCriteria> {
    // Filter rows for this definition
    let rows: Vec<_> = criteria_rows.iter()
        .filter(|r| r.definition_id == definition_id)
        .collect();

    // Find root (parent_id IS NULL)
    let root = rows.iter()
        .find(|r| r.parent_id.is_none())
        .ok_or_else(|| AppError::Internal { ... })?;

    // Build tree recursively
    build_node(root, &rows)
}

fn build_node(row: &OvalCriteriaRow, all_rows: &[&OvalCriteriaRow]) -> OvalCriteriaNode {
    if let Some(test_ref) = &row.criterion_test_ref {
        // Leaf node: criterion
        OvalCriteriaNode::Criterion {
            comment: row.criterion_comment.clone().unwrap_or_default(),
            test_ref: test_ref.clone(),
        }
    } else {
        // Branch node: criteria
        let children: Vec<_> = all_rows.iter()
            .filter(|r| r.parent_id == Some(row.id))
            .sorted_by_key(|r| r.sequence.unwrap_or(0))
            .map(|child| build_node(child, all_rows))
            .collect();
        OvalCriteriaNode::NestedCriteria {
            operator: row.operator.clone().unwrap_or_else(|| "AND".into()),
            children,
        }
    }
}
```

## Performance Considerations

- **Max 5000 definitions per merge** — hard cap to prevent memory issues
- **DB query optimization**: Use `issued_date` composite index for date range queries
- **Batch queries**: Fetch all tests/objects/states for all definition_ids in one query each (not N+1)
- **Streaming output**: For merges with >1000 definitions, use `OvalStreamWriter` from xml module to avoid building full XML in memory

## Test Cases

1. **Empty month**: Query month with no definitions → 404 Not Found
2. **Single definition merge**: 1 definition → output has 1 definition + its components
3. **Multiple definitions, dedup**: 2 definitions sharing same platform test → output has 1 platform test
4. **Date range boundaries**: start=2025-11-01, end=2025-11-30 → includes 11-01 and 11-30
5. **Range exceeds 365 days**: Returns 400 RangeTooLarge error
6. **Start > end**: Returns 400 Validation error
7. **Criteria tree reconstruction**: Criteria with 3 levels (AND→OR→AND) → correct XML nesting
8. **Generator timestamp**: Merged output has current (not original) timestamp in generator

## Acceptance Criteria

- [ ] `merge_by_month(2025, 11)` returns merged XML for all November 2025 definitions
- [ ] Platform detection test (id=...0001) appears only once when shared by multiple definitions
- [ ] Definitions with different oval_ids but identical content are both kept
- [ ] Date range > 365 days returns error
- [ ] Merged XML validates against OVAL schema
- [ ] Performance: 100 definitions merge completes in under 1 second
- [ ] Performance: 1000 definitions merge completes in under 10 seconds
- [ ] Generator in merged output has current timestamp (not from stored data)
