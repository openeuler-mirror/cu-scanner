# 09 - XML Serializer

## Purpose

Serialize OVAL domain models to XML output using `quick-xml` + `serde`. Supports both pretty-printed output (for files) and compact output (for API responses). Also supports streaming output for large merged documents.

## Dependencies

- 01-project-setup (quick-xml, serde)
- 03-error-types
- 04-domain-models (oval.rs with Serialize derives)

## Files to Create

```
src/xml/mod.rs
src/xml/serializer.rs
```

## Reference Design Doc Sections

- Section 6.3 (XML Serialization Design — quick-xml + serde patterns)
- Section 5.2 (OVAL XML output structure)
- Section 14.1 (Streaming output for large merges)

## Public API Surface

```rust
// src/xml/serializer.rs

use crate::error::Result;
use crate::models::oval::OvalDefinitions;

/// Serialize OVAL to a pretty-printed XML string (for CLI files, small outputs).
/// Uses serde + quick_xml::se::Serializer with indent.
pub fn serialize_to_string(oval: &OvalDefinitions, pretty: bool) -> Result<String>;

/// Serialize OVAL to any writer (for streaming/HTTP responses).
/// When pretty=false, no indentation (compact, smaller payload).
/// When pretty=true, uses quick_xml::Writer::new_with_indent for event-based output.
pub fn serialize_to_writer<W: std::io::Write>(oval: &OvalDefinitions, writer: W, pretty: bool) -> Result<()>;

/// Stream-write a merged OVAL document to a writer.
/// Writes root element start, then yields control to caller to write
/// definitions/tests/objects/states one by one, then writes root element end.
/// This avoids loading entire merged XML into memory.
pub struct OvalStreamWriter<W: std::io::Write> {
    writer: quick_xml::Writer<W>,
}

impl<W: std::io::Write> OvalStreamWriter<W> {
    /// Create a new stream writer. Writes XML declaration and root element start tag.
    pub fn new(writer: W, generator: &OvalGenerator, pretty: bool) -> Result<Self>;

    /// Write a single definition element
    pub fn write_definition(&mut self, def: &OvalDefinition) -> Result<()>;

    /// Write a single test/object/state element
    pub fn write_test(&mut self, test: &OvalTest) -> Result<()>;
    pub fn write_object(&mut self, obj: &OvalObject) -> Result<()>;
    pub fn write_state(&mut self, state: &OvalState) -> Result<()>;

    /// Finalize: close root element. Returns the underlying writer.
    pub fn finish(self) -> Result<W>;
}
```

## Implementation Details

### serialize_to_string (serde mode)

```rust
use quick_xml::se::Serializer;

pub fn serialize_to_string(oval: &OvalDefinitions, pretty: bool) -> Result<String> {
    let mut buf = String::new();
    let mut ser = Serializer::new(&mut buf);
    if pretty {
        ser.indent(' ', 2);
    }
    oval.serialize(ser).map_err(|e| AppError::Internal {
        message: format!("XML serialization failed: {}", e),
    })?;
    Ok(buf)
}
```

**IMPORTANT**: `se::Serializer` writer must implement `std::fmt::Write`. Use `String`, NOT `quick_xml::Writer` directly.

### serialize_to_writer (event mode for streaming)

For non-serde streaming to arbitrary `io::Write` writers:

```rust
pub fn serialize_to_writer<W: std::io::Write>(oval: &OvalDefinitions, writer: W, pretty: bool) -> Result<()> {
    let mut xml_writer = if pretty {
        quick_xml::Writer::new_with_indent(writer, b' ', 2)
    } else {
        quick_xml::Writer::new(writer)
    };

    // Write XML declaration
    xml_writer.write_event(Event::Decl(BytesDecl::new("1.0", Some("utf-8"), None)))?;

    // Write root element with all namespace attributes
    let mut root = BytesStart::new("oval_definitions");
    root.push_attribute(("xmlns", oval.xmlns.as_str()));
    root.push_attribute(("xmlns:oval", oval.xmlns_oval.as_str()));
    root.push_attribute(("xmlns:unix-def", oval.xmlns_unix_def.as_str()));
    root.push_attribute(("xmlns:red-def", oval.xmlns_red_def.as_str()));
    root.push_attribute(("xmlns:ind-def", oval.xmlns_ind_def.as_str()));
    root.push_attribute(("xmlns:xsi", oval.xmlns_xsi.as_str()));
    root.push_attribute(("xsi:schemaLocation", oval.schema_location.as_str()));
    xml_writer.write_event(Event::Start(root))?;

    // Write generator (using serde on a nested buffer, then write as raw)
    // ... write each section ...

    xml_writer.write_event(Event::End(BytesEnd::new("oval_definitions")))?;
    Ok(())
}
```

**Note**: Since serde with enum-based heterogeneous lists works for full serialization, prefer `serialize_to_string` for most cases. The streaming writer is for cases where the merged document has thousands of definitions — in that case, manually write events.

## Namespace Constants

```rust
/// Default namespace declarations for OVAL output
pub struct OvalNamespaces {
    pub xmlns: &'static str,           // "http://oval.mitre.org/XMLSchema/oval-definitions-5"
    pub xmlns_oval: &'static str,      // "http://oval.mitre.org/XMLSchema/oval-common-5"
    pub xmlns_unix_def: &'static str,  // "http://oval.mitre.org/XMLSchema/oval-definitions-5#unix"
    pub xmlns_red_def: &'static str,   // "http://oval.mitre.org/XMLSchema/oval-definitions-5#linux"
    pub xmlns_ind_def: &'static str,   // "http://oval.mitre.org/XMLSchema/oval-definitions-5#independent"
    pub xmlns_xsi: &'static str,       // "http://www.w3.org/2001/XMLSchema-instance"
    pub schema_location: &'static str, // full schemaLocation value
}

pub const DEFAULT_NAMESPACES: OvalNamespaces = OvalNamespaces {
    xmlns: "http://oval.mitre.org/XMLSchema/oval-definitions-5",
    xmlns_oval: "http://oval.mitre.org/XMLSchema/oval-common-5",
    xmlns_unix_def: "http://oval.mitre.org/XMLSchema/oval-definitions-5#unix",
    xmlns_red_def: "http://oval.mitre.org/XMLSchema/oval-definitions-5#linux",
    xmlns_ind_def: "http://oval.mitre.org/XMLSchema/oval-definitions-5#independent",
    xmlns_xsi: "http://www.w3.org/2001/XMLSchema-instance",
    schema_location: "http://oval.mitre.org/XMLSchema/oval-definitions-5 oval-definitions-schema.xsd http://oval.mitre.org/XMLSchema/oval-definitions-5#linux linux-definitions-schema.xsd http://oval.mitre.org/XMLSchema/oval-definitions-5#unix unix-definitions-schema.xsd http://oval.mitre.org/XMLSchema/oval-definitions-5#independent independent-definitions-schema.xsd http://oval.mitre.org/XMLSchema/oval-common-5 oval-common-schema.xsd",
};
```

## Test Cases

1. **Pretty output**: Serialize a simple OVAL → verify 2-space indentation
2. **Compact output**: Serialize with `pretty=false` → verify single-line XML (no extra whitespace)
3. **XML declaration**: Output starts with `<?xml version="1.0" encoding="utf-8"?>`
4. **Namespace attributes**: All 6 xmlns attributes present on root element
5. **Enum serialization**: `OvalTest::RpmInfo` → `<red-def:rpminfo_test>...</red-def:rpminfo_test>`
6. **Round-trip XSD validation**: Serialized XML validates against OVAL schema (run `xmllint --schema` in test)
7. **Special characters**: Package descriptions with `<`, `>`, `&` are properly escaped
8. **Empty containers**: OVAL with no tests → `<tests/>` (self-closing)

## Acceptance Criteria

- [ ] `serialize_to_string(&oval, true)` produces XML with 2-space indentation
- [ ] `serialize_to_string(&oval, false)` produces compact XML
- [ ] Output validates against OVAL 5.10 schema (xmllint)
- [ ] All namespace prefixes match the design doc (red-def, unix-def, ind-def, oval)
- [ ] Prefix typos would produce test failures caught by xmllint validation
- [ ] Streaming writer produces identical output to string serializer for the same document
- [ ] Serialization of a 1000-definition document completes in under 5 seconds
