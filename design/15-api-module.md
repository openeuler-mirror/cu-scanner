# 15 - API Module

## Purpose

actix-web HTTP API server: route definitions, request handlers, input validation, rate limiting, audit logging, and error response formatting.

## Dependencies

- 02-config (ServerConfig, AuthConfig)
- 03-error-types
- 05-database (repositories)
- 11-ingest-service (for POST /csaf)
- 12-merge-service (for GET /oval/month, /oval/range)
- 14-auth-module (JWT middleware, handlers)

## Files to Create

```
src/api/mod.rs
src/api/routes.rs        # Route definitions
src/api/handlers.rs      # Request handler implementations
src/api/middleware.rs    # Rate limiting, request logging, audit
src/api/response.rs      # Response formatting helpers
```

## Reference Design Doc Sections

- Section 6.5 (HTTP API module design)
- Section 7 (API interface design)
- Section 9.7 (Input validation)
- Section 9.10 (Audit logging)
- Section 14.1 (Rate limiting)

---

## Route Definitions (`src/api/routes.rs`)

```rust
use actix_web::web;

/// Configure all API routes on the actix-web App.
/// Routes requiring auth are wrapped with the JWT middleware.
pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg
        // Public (no auth required)
        .service(web::resource("/health").route(web::get().to(handlers::health_handler)))
        // Auth routes (no JWT required, but rate-limited)
        .service(
            web::scope("/auth")
                .route("/login", web::post().to(handlers::login_handler))
                .route("/refresh", web::post().to(handlers::refresh_handler))
                .route("/password", web::post().to(handlers::change_password_handler))
        )
        // Protected routes (JWT required)
        .service(
            web::scope("/oval")
                .wrap(auth::middleware::JwtAuthMiddleware)
                .route("/{id}", web::get().to(handlers::get_oval_by_id))
                .route("/month/{year_month}", web::get().to(handlers::get_oval_by_month))
                .route("/range", web::get().to(handlers::get_oval_by_range))
        )
        // Upload route (JWT required, admin role)
        .service(
            web::resource("/csaf")
                .wrap(auth::middleware::JwtAuthMiddleware)
                .route(web::post().to(handlers::upload_csaf_handler))
        )
        // Metrics (internal only, no auth)
        .route("/metrics", web::get().to(handlers::metrics_handler));
}
```

---

## Handlers (`src/api/handlers.rs`)

### GET /health

```
No auth required.
Response: 200 JSON
{
  "status": "ok",
  "version": "1.0.0",
  "database": "connected" | "disconnected",
  "timestamp": "2025-12-01T10:00:00Z",
  "checks": {
    "database": { "status": "up", "latency_ms": 5 },
    "disk": { "status": "up", "free_gb": 45.2 },
    "last_sync": { "status": "up", "time": "2025-12-01T09:00:00Z" }
  }
}
```

```rust
pub async fn health_handler(
    pool: web::Data<AnyPool>,
    app_state: web::Data<Arc<AppState>>,
) -> HttpResponse {
    // Check DB connectivity
    let db_status = sqlx::query("SELECT 1").execute(pool.as_ref()).await;

    let response = HealthResponse {
        status: "ok",
        version: env!("CARGO_PKG_VERSION"),
        database: if db_status.is_ok() { "connected" } else { "disconnected" },
        timestamp: chrono::Utc::now().to_rfc3339(),
        checks: build_health_checks(pool, app_state).await,
    };
    HttpResponse::Ok().json(response)
}
```

### GET /oval/{id}

```
Auth: JWT required
Path param: id = OVAL numeric ID (e.g., "20251665")
Regex: ^\d{8,}$
Response: 200 XML (single OVAL definition)
          404 JSON (not found)
```

```rust
pub async fn get_oval_by_id(
    pool: web::Data<AnyPool>,
    path: web::Path<String>,
    query: web::Query<OvalQuery>,
) -> HttpResponse {
    let numeric_id = path.into_inner();

    // Validate: pure digits, at least 8 chars
    if !numeric_id.chars().all(|c| c.is_ascii_digit()) || numeric_id.len() < 8 {
        return AppError::Validation {
            message: "Invalid OVAL ID format".into(),
            field: Some("id".into()),
        }.error_response();
    }

    // Build full OVAL ID and query
    let oval_id = format!("oval:com.chinaunicom.cuos:def:{}", numeric_id);

    match OvalDefinitionRepository::new(pool.get_ref().clone())
        .find_latest_by_oval_id(&oval_id).await
    {
        Ok(Some(def)) => {
            // Fetch components and build single OVAL XML
            match build_single_oval_xml(pool.get_ref(), &def).await {
                Ok(xml) => HttpResponse::Ok()
                    .content_type("application/xml")
                    .body(xml),
                Err(e) => e.error_response(),
            }
        }
        Ok(None) => AppError::NotFound {
            resource: "OVAL definition".into(),
            id: numeric_id,
        }.error_response(),
        Err(e) => e.error_response(),
    }
}
```

### GET /oval/month/{year-month}

```
Auth: JWT required
Path param: year-month = "YYYY-MM"
Regex: ^\d{4}-\d{2}$
Response: 200 XML (merged OVAL)
          404 JSON (no data for month)
```

### GET /oval/range?start=YYYY-MM-DD&end=YYYY-MM-DD

```
Auth: JWT required
Query params:
  start: YYYY-MM-DD (required)
  end:   YYYY-MM-DD (required)
Validation:
  - Date format must be YYYY-MM-DD
  - start <= end
  - span <= 365 days
Response: 200 XML (merged OVAL)
          400 JSON (range too large, format error)
          404 JSON (no data)
```

### POST /csaf

```
Auth: JWT required (admin role recommended)
Content-Type: application/json  OR  multipart/form-data
Body: CSAF JSON (for json) or file field (for multipart)
Query params:
  force: bool (default: false)
  dry_run: bool (default: false)
Response: 201/200 JSON (IngestResult)
          400/413/422/409/500 JSON (errors)
```

```rust
pub async fn upload_csaf_handler(
    pool: web::Data<AnyPool>,
    ingest_service: web::Data<Arc<IngestService>>,
    claims: Claims,
    query: web::Query<UploadQuery>,
    body: web::Bytes,  // For JSON body
    // For multipart, use actix_multipart::Multipart
) -> HttpResponse {
    // 1. Check role (admin required for write operations)
    if claims.role != "admin" {
        return AppError::Unauthorized { message: "Admin role required".into() }.error_response();
    }

    // 2. Size check (>10MB → 413)
    if body.len() > 10 * 1024 * 1024 {
        return AppError::PayloadTooLarge { ... }.error_response();
    }

    // 3. Delegate to ingest_service
    match ingest_service.ingest_from_bytes(&body, query.force, query.dry_run).await {
        Ok(result) => {
            let status = match result.action {
                IngestAction::Created => StatusCode::CREATED,
                _ => StatusCode::OK,
            };
            HttpResponse::build(status).json(result)
        }
        Err(e) => e.error_response(),
    }
}
```

---

## Middleware (`src/api/middleware.rs`)

### Rate Limiter

```rust
use governor::{Quota, RateLimiter, clock::DefaultClock, state::InMemoryState, middleware::NoOpMiddleware};
use std::collections::HashMap;

/// Per-route rate limiter
pub struct RateLimitMiddleware {
    // Map of route pattern → rate limiter
    limiters: HashMap<String, RateLimiter<String, InMemoryState, DefaultClock, NoOpMiddleware>>,
}

impl RateLimitMiddleware {
    pub fn new() -> Self {
        let mut limiters = HashMap::new();
        limiters.insert("/auth/login".into(),
            RateLimiter::keyed(Quota::per_minute(5.try_into().unwrap()))); // 5/min/IP
        limiters.insert("/csaf".into(),
            RateLimiter::keyed(Quota::per_minute(30.try_into().unwrap()))); // 30/min/IP
        limiters.insert("default".into(),
            RateLimiter::keyed(Quota::per_minute(100.try_into().unwrap()))); // 100/min/IP
        Self { limiters }
    }

    pub fn check(&self, path: &str, ip: &str) -> Result<(), AppError> {
        let limiter = self.limiters.get(path)
            .unwrap_or_else(|| self.limiters.get("default").unwrap());

        match limiter.check_key(&ip.to_string()) {
            Ok(_) => Ok(()),
            Err(_) => Err(AppError::Unauthorized {
                message: "Rate limit exceeded".into(),
            }),
        }
    }
}
```

### Request Audit Logger

```rust
/// Logs each API request for audit trail.
/// Sensitive endpoints (/auth/login, /auth/password, POST /csaf) get full audit.
pub async fn audit_middleware(
    req: ServiceRequest,
    ...
) -> Result<ServiceResponse, Error> {
    let start = std::time::Instant::now();
    let method = req.method().clone();
    let path = req.path().to_string();
    let ip = req.peer_addr().map(|a| a.ip().to_string()).unwrap_or_default();
    let request_id = req.headers().get("X-Request-ID")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();
    let user = req.extensions().get::<Claims>().map(|c| c.sub.clone());

    let resp = next.call(req).await?;
    let duration_ms = start.elapsed().as_millis();

    // Log audit record
    tracing::info!(
        target: "audit",
        method = %method,
        path = %path,
        status = resp.status().as_u16(),
        duration_ms = duration_ms,
        ip = %ip,
        user = ?user,
        request_id = %request_id,
    );

    Ok(resp)
}
```

---

## Response Helpers (`src/api/response.rs`)

```rust
/// Build a JSON success response
pub fn json_ok<T: Serialize>(data: &T) -> HttpResponse {
    HttpResponse::Ok().json(data)
}

/// Set security headers on every response
pub fn add_security_headers(resp: &mut HttpResponseBuilder) {
    resp.insert_header(("X-Content-Type-Options", "nosniff"));
    resp.insert_header(("X-Frame-Options", "DENY"));
    resp.insert_header(("Content-Security-Policy", "default-src 'none'"));
    resp.insert_header(("Strict-Transport-Security", "max-age=31536000; includeSubDomains"));
}
```

## Test Cases

1. **GET /health**: Returns 200 with status info, no auth required
2. **GET /oval/20251665**: Returns OVAL XML, requires valid JWT
3. **GET /oval/20251665 without auth**: Returns 401 JSON error
4. **GET /oval/month/2025-11**: Returns merged XML for November 2025
5. **GET /oval/range?start=2025-01-01&end=2025-12-31**: Returns merged XML
6. **GET /oval/range?start=2025-01-01&end=2026-12-31**: Returns 400 range too large
7. **POST /csaf**: Upload valid CSAF → returns 201 with IngestResult
8. **POST /csaf with invalid JSON**: Returns 400 INVALID_JSON
9. **POST /auth/login**: Valid credentials → returns token
10. **Rate limit**: 6 login attempts in 1 minute → 6th returns 429/401

## Acceptance Criteria

- [ ] `/health` returns health status without authentication
- [ ] `/oval/*` endpoints require valid JWT
- [ ] `/csaf` requires admin role JWT
- [ ] Date range validation: ≤365 days, start ≤ end
- [ ] Rate limiting: 5/min on login, 30/min on upload, 100/min default
- [ ] All responses have security headers
- [ ] Audit logs are written for all requests
- [ ] XML responses have `Content-Type: application/xml`
- [ ] JSON error responses have `Content-Type: application/json`
- [ ] CORS is configured with specific allowed origins (no wildcard)
