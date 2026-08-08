# 00 - Architecture Overview

## System Summary

cu-scanner is a Rust tool/service that converts CSAF (Common Security Advisory Framework) JSON security advisories into OVAL (Open Vulnerability and Assessment Language) XML patch definitions. It supports CLI mode (single/batch conversion) and Server mode (HTTP API with database persistence).

## Module Dependency Graph

```
                         ┌──────────────────┐
                         │   16-server-main │ ◄── Entry point
                         └────────┬─────────┘
                                  │
              ┌───────────────────┼───────────────────┐
              │                   │                   │
     ┌────────┴────────┐ ┌────────┴───────┐ ┌─────────┴───────┐
     │   13-cli-module │ │  15-api-module │ │    (cron sync)  │
     └────────┬────────┘ └────────┬───────┘ └─────────┬───────┘
              │                   │                   │
              └───────────────────┼───────────────────┘
                                  │
              ┌───────────────────┼───────────────────┐
              │                   │                   │
     ┌────────┴────────┐ ┌────────┴────────┐ ┌────────┴────────┐
     │ 14-auth-module  │ │11-ingest-service│ │ 12-merge-service│
     └────────┬────────┘ └────────┬────────┘ └────────┬────────┘
              │                   │                   │
              │          ┌────────┴────────┐          │
              │          │                 │          │
              │   ┌──────┴──────┐  ┌───────┴─────┐    │
              │   │10-download  │  │08-conversion│    │
              │   │  -service   │  │  -engine    │    │
              │   └──────┬──────┘  └──────┬──────┘    │
              │          │                │           │
              │          │   ┌────────────┼──────┐    │
              │          │   │            │      │    │
              │   ┌──────┴───┴──┐ ┌───────┴──┐ ┌─┴────┴──────┐
              │   │06-csaf      │ │07-epoch  │ │09-xml       │
              │   │  -parser    │ │-resolver │ │  -serializer│
              │   └──────┬──────┘ └────┬─────┘ └───────┬─────┘
              │          │             │               │
              └──────────┼─────────────┼───────────────┘
                         │             │
              ┌──────────┴─────────────┴───────────────┐
              │            05-database                 │
              └──────────────┬─────────────────────────┘
                             │
              ┌──────────────┼────────────────┐
              │              │                │
     ┌────────┴──────┐ ┌─────┴─────┐ ┌────────┴──────┐
     │04-domain      │ │03-error   │ │ 02-config     │
     │  -models      │ │  -types   │ │               │
     └───────────────┘ └───────────┘ └───────────────┘
                             │
                    ┌────────┴──────┐
                    │01-project     │
                    │  -setup       │
                    └───────────────┘
```

## Implementation Order (Recommended)

Implement modules in this order. Each module only depends on modules listed before it:

| Phase | Module | Depends On |
|-------|--------|------------|
| **Foundation** | 01-project-setup | - |
| | 02-config | 01 |
| | 03-error-types | 01 |
| | 04-domain-models | 01 |
| **Data** | 05-database | 02, 03, 04 |
| **Parsing** | 06-csaf-parser | 03, 04 |
| | 07-epoch-resolver | 02, 03 |
| **Core** | 08-conversion-engine | 03, 04, 07 |
| | 09-xml-serializer | 03, 04 |
| **Services** | 10-download-service | 02, 03 |
| | 11-ingest-service | 05, 06, 08, 10 |
| | 12-merge-service | 05, 09 |
| **Interfaces** | 13-cli-module | 02, 08, 09, 11, 12 |
| | 14-auth-module | 02, 03, 05 |
| | 15-api-module | 02, 03, 05, 11, 12, 14 |
| | 16-server-main | All above |

## Source Directory Structure

```
cu-scanner/
├── Cargo.toml
├── config.toml                  # Example config
├── migrations/                  # SQL migration files
│   ├── 20250101000001_initial.sql
│   └── ...
├── design/                      # Module design specs (this directory)
└── src/
    ├── main.rs                  # Entry point
    ├── lib.rs                   # Library root
    ├── config.rs                # Configuration
    ├── error.rs                 # Error types
    ├── models/
    │   ├── mod.rs
    │   ├── csaf.rs              # CSAF JSON models
    │   ├── oval.rs              # OVAL XML models
    │   └── db.rs                # Database row models
    ├── db/
    │   ├── mod.rs
    │   ├── pool.rs              # Connection pool
    │   ├── migrate.rs           # Migration runner
    │   └── repository/
    │       ├── mod.rs
    │       ├── csaf_source.rs
    │       ├── oval_definition.rs
    │       ├── oval_test.rs
    │       ├── oval_object.rs
    │       ├── oval_state.rs
    │       ├── oval_criteria.rs
    │       ├── download_task.rs
    │       ├── user.rs
    │       └── epoch_cache.rs
    ├── parser/
    │   ├── mod.rs
    │   └── csaf.rs              # CSAF parser
    ├── epoch/
    │   ├── mod.rs
    │   └── resolver.rs          # Epoch resolver
    ├── engine/
    │   ├── mod.rs
    │   ├── id_gen.rs            # OVAL ID generator
    │   └── converter.rs         # CSAF → OVAL conversion
    ├── xml/
    │   ├── mod.rs
    │   └── serializer.rs        # OVAL XML serializer
    ├── download/
    │   ├── mod.rs
    │   └── service.rs           # Download service
    ├── ingest/
    │   ├── mod.rs
    │   └── service.rs           # Ingest pipeline
    ├── merge/
    │   ├── mod.rs
    │   └── service.rs           # Merge service
    ├── cli/
    │   ├── mod.rs
    │   └── commands.rs          # CLI subcommands
    ├── auth/
    │   ├── mod.rs
    │   ├── jwt.rs               # JWT token handling
    │   ├── middleware.rs        # Auth middleware
    │   └── handlers.rs          # Login/refresh/password
    ├── api/
    │   ├── mod.rs
    │   ├── routes.rs            # Route definitions
    │   ├── handlers.rs          # Request handlers
    │   ├── middleware.rs        # API middleware (rate limit, logging)
    │   └── response.rs          # Response formatting
    └── server/
        ├── mod.rs
        └── app.rs               # Server startup
```

## Key Design Decisions (from design doc Section 12)

| Decision | Choice |
|----------|--------|
| ORM | **sqlx** (compile-time checked, async, multi-DB) |
| DB storage | **Metadata split storage** (no full XML text) |
| Merge dedup | By **OVAL ID** on tests/objects/states |
| Incremental sync | Overwrite local if `tracking.version` is newer |
| XML library | **quick-xml 0.38** + **serde** (namespace literal passthrough) |
| API auth | **JWT (Bearer Token)** with `jsonwebtoken` crate |
| Password hash | **bcrypt** (cost ≥ 12) |
| Epoch source | yum/dnf query with DB cache fallback |
| Platform detection | `rpmverifyfile_test` on `/etc/os-release` |
| Sequence numbers | 4-digit (0001-9999), platform 0001-0100, signature 0101-0200, version 0201-9999 |
