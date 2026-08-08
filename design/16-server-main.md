# 16 - Server Main

## Purpose

Server startup orchestration: load config, initialize database, set up all services, configure actix-web with middleware, start background sync scheduler, handle graceful shutdown and SIGHUP config reload.

## Dependencies

- All previous modules (this is the final integration module)

## Files to Create

```
src/server/mod.rs
src/server/app.rs
```

## Reference Design Doc Sections

- Section 4 (System architecture)
- Section 6.1 (Server subcommand)
- Section 6.5 (HTTP API module)
- Section 13.9 (Config reload / SIGHUP)
- Section 14.2 (Circuit breaker, degradation)
- Section 14.3 (Health check, monitoring)

## Public API Surface

```rust
// src/server/mod.rs

pub use app::run_server;

// src/server/app.rs

use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::error::Result;

/// Application state shared across all actix-web workers
pub struct AppState {
    pub config: Arc<RwLock<Config>>,
    pub pool: AnyPool,
    pub merge_service: Arc<MergeService>,
    pub ingest_service: Arc<IngestService>,
    pub auth_service: Option<Arc<JwtAuth>>,
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub last_sync_time: Arc<RwLock<Option<chrono::DateTime<chrono::Utc>>>>,
}

/// Entry point for `cu-scanner server --config <path>`.
pub async fn run_server(config_path: &Path) -> Result<()>;
```

## Startup Sequence

```rust
pub async fn run_server(config_path: &Path) -> Result<()> {
    // === PHASE 1: Load Configuration ===
    let mut config = Config::load(config_path)?;
    config.resolve_secrets();
    let config = Arc::new(RwLock::new(config));

    {
        let cfg = config.read().await;
        // Initialize logging based on config
        init_logging(&cfg.logging)?;
        tracing::info!("Starting cu-scanner v{}", env!("CARGO_PKG_VERSION"));
        tracing::info!("Config loaded from: {}", config_path.display());
    }

    // === PHASE 2: Initialize Database ===
    let cfg = config.read().await;
    let pool = db::pool::create_pool(&cfg.database).await?;
    db::migrate::run_migrations(&pool).await?;
    tracing::info!("Database connected: {}", cfg.database.driver);

    // === PHASE 3: Bootstrap Admin (if auth enabled) ===
    if cfg.auth.enabled {
        auth::bootstrap_admin(&pool, &cfg.auth).await?;
    }

    // === PHASE 4: Build Services ===
    let oval_cfg = cfg.oval.clone();
    let yum_cfg = cfg.yum.clone();
    let sync_cfg = cfg.sync.clone();

    // Epoch resolver
    let db_epoch_repo = if yum_cfg.cache_to_db {
        Some(EpochCacheRepository::new(pool.clone()))
    } else {
        None
    };
    let epoch_resolver = YumEpochResolver::new(yum_cfg, db_epoch_repo);

    // Converter (server mode)
    let converter = Converter::new(oval_cfg.clone(), epoch_resolver);

    // Download service
    let download_service = DownloadService::new(sync_cfg.clone())?;

    // Ingest service
    let ingest_service = IngestService::new(
        pool.clone(),
        converter,
        Some(download_service),
        IngestConfig {
            concurrent_limit: sync_cfg.concurrent_limit,
            max_retries: sync_cfg.max_retries,
            retry_interval_sec: sync_cfg.retry_interval_sec,
        },
    );

    // Merge service
    let merge_service = MergeService::new(pool.clone(), oval_cfg.clone());

    // Auth service (if enabled)
    let auth_service = if cfg.auth.enabled {
        Some(Arc::new(JwtAuth::new(&cfg.auth)?))
    } else {
        None
    };

    // === PHASE 5: Build App State ===
    let app_state = Arc::new(AppState {
        config: config.clone(),
        pool: pool.clone(),
        merge_service: Arc::new(merge_service),
        ingest_service: Arc::new(ingest_service),
        auth_service: auth_service.clone(),
        start_time: chrono::Utc::now(),
        last_sync_time: Arc::new(RwLock::new(None)),
    });
    drop(cfg); // release config read lock

    // === PHASE 6: Optional Startup Sync ===
    {
        let cfg = config.read().await;
        if cfg.sync.on_startup {
            tracing::info!("Running startup sync...");
            match app_state.ingest_service.run_full_sync().await {
                Ok(report) => {
                    tracing::info!("Startup sync complete: {} created, {} updated, {} skipped, {} failed",
                        report.created, report.updated, report.skipped, report.failed);
                    *app_state.last_sync_time.write().await = Some(chrono::Utc::now());
                }
                Err(e) => {
                    tracing::warn!("Startup sync failed: {}", e);
                }
            }
        }
    }

    // === PHASE 7: Start HTTP Server ===
    let server_cfg = {
        let cfg = config.read().await;
        cfg.server.clone()
    };

    let bind_addr = format!("{}:{}", server_cfg.host, server_cfg.port);
    tracing::info!("Starting HTTP server on {}", bind_addr);

    let server = actix_web::HttpServer::new(move || {
        build_app(
            app_state.clone(),
            auth_service.clone(),
        )
    })
    .workers(server_cfg.workers)
    .bind(&bind_addr)?
    .run();

    // === PHASE 8: Start Cron Scheduler (if configured) ===
    let cron_handle = {
        let cfg = config.read().await;
        if let Some(ref cron_expr) = cfg.sync.cron {
            let ingest = app_state.ingest_service.clone();
            let last_sync = app_state.last_sync_time.clone();
            tracing::info!("Starting sync cron: {}", cron_expr);

            let scheduler = tokio_cron_scheduler::JobScheduler::new().await?;
            let job = tokio_cron_scheduler::Job::new_async(cron_expr, move |_uuid, _lock| {
                let ingest = ingest.clone();
                let last_sync = last_sync.clone();
                Box::pin(async move {
                    tracing::info!("Cron sync triggered");
                    match ingest.run_full_sync().await {
                        Ok(report) => {
                            tracing::info!("Cron sync complete: {} created, {} failed",
                                report.created, report.failed);
                            *last_sync.write().await = Some(chrono::Utc::now());
                        }
                        Err(e) => tracing::error!("Cron sync failed: {}", e),
                    }
                })
            })?;
            scheduler.add(job).await?;
            scheduler.start().await?;
            Some(scheduler)
        } else {
            None
        }
    };

    // === PHASE 9: Signal Handling ===
    let server_handle = server.handle();
    tokio::spawn(async move {
        handle_signals(server_handle.clone(), config).await;
    });

    // === PHASE 10: Run ===
    tracing::info!("cu-scanner is ready");
    server.await?;

    Ok(())
}
```

## Application Builder

```rust
fn build_app(
    app_state: Arc<AppState>,
    auth_service: Option<Arc<JwtAuth>>,
) -> actix_web::App<
    impl actix_web::dev::ServiceFactory<
        actix_web::dev::ServiceRequest,
        Config = (),
        Error = actix_web::Error,
        InitError = (),
    >,
> {
    actix_web::App::new()
        // Shared state
        .app_data(web::Data::from(app_state.pool.clone()))
        .app_data(web::Data::from(app_state.merge_service.clone()))
        .app_data(web::Data::from(app_state.ingest_service.clone()))
        .app_data(web::Data::from(app_state.clone()))
        // Auth middleware (if enabled)
        .app_data(web::Data::from(auth_service.clone()))
        // Global middleware
        .wrap(api::middleware::audit_middleware())
        .wrap(api::middleware::rate_limit_middleware())
        .wrap(cors_configuration())
        .wrap(tracing_actix_web::TracingLogger::default())
        // Security headers on all responses
        .wrap(actix_web::middleware::DefaultHeaders::new()
            .add(("X-Content-Type-Options", "nosniff"))
            .add(("X-Frame-Options", "DENY"))
            .add(("Referrer-Policy", "no-referrer")))
        // Routes
        .configure(api::routes::configure_routes)
}
```

## Signal Handling

```rust
async fn handle_signals(
    server_handle: actix_web::dev::ServerHandle,
    config: Arc<RwLock<Config>>,
) {
    use tokio::signal;

    #[cfg(unix)]
    {
        use tokio::signal::unix::{signal, SignalKind};

        let mut sighup = signal(SignalKind::hangup()).expect("SIGHUP handler");
        let mut sigterm = signal(SignalKind::terminate()).expect("SIGTERM handler");
        let mut sigint = signal(SignalKind::interrupt()).expect("SIGINT handler");

        loop {
            tokio::select! {
                _ = sighup.recv() => {
                    tracing::info!("Received SIGHUP, reloading configuration...");
                    match reload_config(config.clone()).await {
                        Ok(()) => tracing::info!("Configuration reloaded successfully"),
                        Err(e) => tracing::error!("Failed to reload config: {}", e),
                    }
                }
                _ = sigterm.recv() => {
                    tracing::info!("Received SIGTERM, shutting down gracefully...");
                    server_handle.stop(true).await;
                    break;
                }
                _ = sigint.recv() => {
                    tracing::info!("Received SIGINT, shutting down gracefully...");
                    server_handle.stop(true).await;
                    break;
                }
            }
        }
    }

    #[cfg(not(unix))]
    {
        // Windows: only handle Ctrl+C
        signal::ctrl_c().await.expect("Ctrl+C handler");
        tracing::info!("Received Ctrl+C, shutting down gracefully...");
        server_handle.stop(true).await;
    }
}
```

## Config Reload (SIGHUP)

```rust
async fn reload_config(config: Arc<RwLock<Config>>) -> Result<()> {
    // Re-read the config file
    let path = {
        let cfg = config.read().await;
        // Store the original path in AppState...
        // For simplicity, re-derive from environment or use a stored path
        std::env::var("CU_SCANNER_CONFIG")
            .unwrap_or_else(|_| "config.toml".to_string())
    };

    let mut new_config = Config::load(Path::new(&path))?;
    new_config.resolve_secrets();

    // Atomic swap
    let mut cfg = config.write().await;
    *cfg = new_config;

    tracing::info!("Configuration reloaded from {}", path);
    Ok(())
}
```

## CORS Configuration

```rust
fn cors_configuration() -> actix_cors::Cors {
    actix_cors::Cors::default()
        .allowed_origin("https://internal.cucloud.com")
        .allowed_methods(vec!["GET", "POST"])
        .allowed_headers(vec!["Authorization", "Content-Type", "X-Request-ID"])
        .max_age(3600)
}
```

## Database Circuit Breaker

When the database is unavailable, the API returns 503 instead of panicking:

```rust
async fn check_db_health(pool: &AnyPool) -> bool {
    sqlx::query("SELECT 1")
        .execute(pool)
        .await
        .is_ok()
}
```

The `/health` endpoint reports DB status; other endpoints check before operations.

## Test Cases

1. **Server startup**: `server -c config.toml` → binds to configured port
2. **Health check**: `GET /health` returns healthy status
3. **Startup sync**: `sync.on_startup = true` → runs sync before accepting requests
4. **Config reload**: SIGHUP → config file re-read, values updated
5. **Graceful shutdown**: SIGTERM → stops accepting new requests, finishes in-flight
6. **DB unavailable**: Stop DB → /health reports disconnected, /oval returns 503
7. **CORS**: Request from non-allowed origin → CORS rejection
8. **Cron sync**: Cron expression fires at expected time

## Acceptance Criteria

- [ ] Server starts and binds to configured host:port
- [ ] `/health` returns 200 with full health details
- [ ] Database migrations run automatically on startup
- [ ] Bootstrap admin created if users table is empty
- [ ] SIGHUP reloads config without dropping connections
- [ ] SIGTERM/SIGINT triggers graceful shutdown (in-flight requests complete)
- [ ] Cron scheduler fires sync on configured schedule
- [ ] CORS allows only configured origins
- [ ] Server starts even if initial sync fails (doesn't crash)
- [ ] `workers` config controls number of actix-web worker threads
- [ ] Logging level can be changed at runtime via config reload
