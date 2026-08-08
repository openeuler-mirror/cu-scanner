# 13 - CLI Module

## Purpose

Command-line interface using `clap` with subcommands: `convert`, `server`, `sync`, and `user`. This is the primary entry point for both CLI mode and launching the server.

## Dependencies

- 02-config
- 03-error-types
- 08-conversion-engine (Converter with ConversionMode::Cli)
- 09-xml-serializer
- 11-ingest-service (for sync command)
- 12-merge-service (not directly, but server uses it)
- 14-auth-module (for `user` subcommand)
- 15-api-module (for `server` subcommand)

## Files to Create

```
src/cli/mod.rs
src/cli/commands.rs
```

## Reference Design Doc Sections

- Section 6.1 (CLI module — subcommands and parameters)
- Section 13.8 (User management CLI)

## Public API Surface

```rust
// src/cli/mod.rs

use clap::{Parser, Subcommand};
use crate::error::Result;

/// Run the CLI, parse args, dispatch to subcommand.
pub async fn run() -> Result<()>;

// src/cli/commands.rs

use clap::Parser;

/// cu-scanner: CSAF to OVAL converter
#[derive(Parser)]
#[command(name = "cu-scanner", version, about)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Convert CSAF JSON file(s) to OVAL XML file(s)
    Convert(ConvertArgs),

    /// Start the HTTP API server
    Server(ServerArgs),

    /// Run a one-time sync from remote CSAF source
    Sync(SyncArgs),

    /// Manage API users (admin only, runs locally on server)
    User(UserArgs),
}
```

## Subcommand: `convert`

```rust
#[derive(clap::Args)]
pub struct ConvertArgs {
    /// Input CSAF JSON file or directory
    #[arg(short = 'i', long = "input")]
    pub input: PathBuf,

    /// Output OVAL XML file or directory
    #[arg(short = 'o', long = "output")]
    pub output: PathBuf,

    /// Force overwrite existing output files
    #[arg(short = 'f', long = "force", default_value_t = false)]
    pub force: bool,

    /// Enable verbose (DEBUG level) logging
    #[arg(short = 'v', long = "verbose", default_value_t = false)]
    pub verbose: bool,

    /// Path to config.toml (optional, for yum config)
    #[arg(short = 'c', long = "config")]
    pub config: Option<PathBuf>,

    /// Comma-separated epoch overrides: "pkg1=0,pkg2=1"
    #[arg(long = "epoch-map")]
    pub epoch_map: Option<String>,
}
```

**Behavior**:
1. If `--input` is file → convert single file → output to `--output` (or `--output` dir + auto-named)
2. If `--input` is dir → traverse `*.json` files → convert each → output to `--output` dir
3. Auto-create output directory if missing
4. Existing files: skip (print message) unless `--force`
5. Output naming: `csaf-cuos-sa-2025-1665.json` → `cuos-{first_pkg_name}-20251665.oval.xml`
6. Always uses `ConversionMode::Cli` (no signature checks)
7. Epoch resolution: if `--epoch-map` provided, use it; otherwise try yum config if `--config` given; default to `"0"`

## Subcommand: `server`

```rust
#[derive(clap::Args)]
pub struct ServerArgs {
    /// Path to TOML configuration file
    #[arg(short = 'c', long = "config")]
    pub config: PathBuf,
}
```

**Behavior**:
1. Load config from TOML file
2. Initialize database pool, run migrations
3. Bootstrap admin user if needed
4. Start actix-web server on configured host:port
5. Optionally run initial sync (if `sync.on_startup = true`)
6. Optionally start cron scheduler (if `sync.cron` is set)
7. Handle SIGHUP for config reload, SIGTERM for graceful shutdown

## Subcommand: `sync`

```rust
#[derive(clap::Args)]
pub struct SyncArgs {
    /// Path to TOML configuration file
    #[arg(short = 'c', long = "config")]
    pub config: PathBuf,
}
```

**Behavior**:
1. Load config
2. Initialize DB pool
3. Run `IngestService::run_full_sync()` once
4. Print SyncReport and exit
5. Does NOT start HTTP server or cron

## Subcommand: `user`

```rust
#[derive(clap::Args)]
pub struct UserArgs {
    #[command(subcommand)]
    pub command: UserCommand,
}

#[derive(Subcommand)]
pub enum UserCommand {
    /// Add a new API user (prompts for password interactively)
    Add {
        #[arg(long)]
        username: String,
        #[arg(long, default_value = "user")]
        role: String,           // "admin" | "user"
    },
    /// List all API users
    List,
    /// Reset a user's password (prompts for new password)
    Passwd {
        #[arg(long)]
        username: String,
    },
    /// Disable a user account
    Disable {
        #[arg(long)]
        username: String,
    },
    /// Enable a user account
    Enable {
        #[arg(long)]
        username: String,
    },
}
```

**Behavior** (from design doc Section 13.8):
- Runs on server machine, trust boundary is OS level
- No admin password required (can already access DB/shell if they can run CLI on server)
- Records audit log: OS username (`whoami`), timestamp, target user, operation
- `user add`: prompt for password (no echo), bcrypt hash, insert to `users`
- `user passwd`: prompt for new password, bcrypt hash, set `must_change_password = true`
- `user list`: query all users, print table (hide password_hash)
- `user disable/enable`: update user status
- Password policy enforcement: ≥12 chars, zxcvbn check, not in recent 5

## Implementation

### Dispatch Logic (`src/cli/mod.rs`)

```rust
pub async fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Convert(args) => cmd_convert(args).await,
        Commands::Server(args) => cmd_server(args).await,
        Commands::Sync(args) => cmd_sync(args).await,
        Commands::User(args) => cmd_user(args).await,
    }
}

async fn cmd_convert(args: ConvertArgs) -> Result<()> {
    // 1. Setup logging (verbose → DEBUG, else INFO)
    // 2. Load config if --config provided (for yum settings)
    // 3. Create Converter with ConversionMode::Cli
    // 4. If --input is file: convert single
    // 5. If --input is dir: collect *.json, convert each
    // 6. Handle --epoch-map if provided
    // 7. Print summary
}

async fn cmd_server(args: ServerArgs) -> Result<()> {
    // Delegate to server module (Module 16)
    crate::server::run_server(&args.config).await
}

async fn cmd_sync(args: SyncArgs) -> Result<()> {
    // 1. Load config
    // 2. Init DB pool, run migrations
    // 3. Create IngestService + DownloadService
    // 4. Run sync
    // 5. Print SyncReport
}

async fn cmd_user(args: UserArgs) -> Result<()> {
    // 1. Load config (for DB connection)
    // 2. Connect to DB
    // 3. Dispatch to user command handler
    // 4. Record audit log
}
```

## Output Naming for `convert`

```rust
fn output_filename(csaf_id: &str, first_package_name: &str) -> String {
    let numeric_id = extract_oval_numeric_id(csaf_id);
    format!("cuos-{}-{}.oval.xml", first_package_name, numeric_id)
}
// "CUOS-SA-2025-1665" + "openssh" → "cuos-openssh-20251665.oval.xml"
// "CUOS-SA-2025-1665" + "openssh-askpass" → "cuos-openssh-askpass-20251665.oval.xml"
```

## Test Cases

1. **Convert single file**: `cargo run -- convert -i test.json -o out.xml` → creates out.xml
2. **Convert directory**: `cargo run -- convert -i ./csaf/ -o ./oval/` → converts all .json files
3. **Force overwrite**: `--force` overwrites existing output file
4. **Skip existing**: Without `--force`, existing file prints "Skipping ... already exists"
5. **Invalid input**: Non-existent path → error message
6. **User add**: `cargo run -- user add --username ops01 --role user` → prompts password, creates user
7. **User list**: `cargo run -- user list` → prints table
8. **Password policy**: Password < 12 chars → rejected with message

## Acceptance Criteria

- [ ] `convert --input file.json --output out.xml` produces valid OVAL XML
- [ ] `convert --input dir/ --output outdir/` processes all .json files
- [ ] `server --config config.toml` starts HTTP server (verified by /health)
- [ ] `sync --config config.toml` runs sync and prints report
- [ ] `user add` creates user with bcrypt-hashed password
- [ ] `user list` shows users without password hashes
- [ ] `user disable/enable` changes user status
- [ ] Output naming follows the `cuos-{pkg}-{id}.oval.xml` convention
- [ ] All subcommands print meaningful error messages on failure
