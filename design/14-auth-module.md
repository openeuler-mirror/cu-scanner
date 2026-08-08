# 14 - Auth Module

## Purpose

JWT-based authentication for the HTTP API. Includes token generation/validation, user management, password hashing, actix-web middleware, and login/refresh/password change handlers.

## Dependencies

- 02-config (AuthConfig)
- 03-error-types
- 05-database (UserRepository)

## Files to Create

```
src/auth/mod.rs
src/auth/jwt.rs           # JWT token generation/verification
src/auth/middleware.rs    # actix-web auth middleware
src/auth/handlers.rs      # login, refresh, password change handlers
```

## Reference Design Doc Sections

- Chapter 13 (API Authentication — JWT)
- Section 5.3 (users table)
- Section 6.1 (user CLI subcommand)
- Section 9.9 (JWT Secret rotation, Token revocation)

---

## 14a. JWT Token Handling (`src/auth/jwt.rs`)

```rust
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};
use serde::{Serialize, Deserialize};
use uuid::Uuid;

/// JWT Claims payload
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    /// Subject (username)
    pub sub: String,
    /// JWT ID (UUID) — used for blacklist/revocation
    pub jti: String,
    /// Issued at (Unix timestamp)
    pub iat: usize,
    /// Expiration (Unix timestamp)
    pub exp: usize,
    /// User role: "admin" | "user"
    pub role: String,
}

/// JWT authentication service
pub struct JwtAuth {
    /// Primary signing key
    encoding_key: EncodingKey,
    /// Primary verification key
    decoding_key: DecodingKey,
    /// Secondary verification key (for secret rotation)
    secondary_decoding_key: Option<DecodingKey>,
    /// Token expiration in hours
    expiration_hours: u64,
    /// Refresh token expiration in hours
    refresh_expiration_hours: u64,
    /// Blacklist (in-memory LRU cache)
    blacklist: Arc<Mutex<LruCache<String, usize>>>, // jti → exp
}

impl JwtAuth {
    /// Create from config. If primary/secondary secrets configured, use them;
    /// otherwise fall back to jwt_secret.
    pub fn new(config: &AuthConfig) -> Result<Self>;

    /// Generate an access token for a user.
    pub fn generate_token(&self, username: &str, role: &str) -> Result<String>;

    /// Generate a refresh token (longer expiry).
    pub fn generate_refresh_token(&self, username: &str, role: &str) -> Result<String>;

    /// Verify a token and return the Claims.
    /// Tries primary key first, then secondary (for rotation).
    /// Also checks blacklist.
    pub fn verify_token(&self, token: &str) -> Result<Claims>;

    /// Add a token's jti to the blacklist (for logout/revocation).
    pub fn blacklist_token(&self, jti: &str, exp: usize);

    /// Check if a jti is in the blacklist.
    pub fn is_blacklisted(&self, jti: &str) -> bool;
}
```

**HS256 algorithm** (HMAC-SHA256). Algorithm constant: `jsonwebtoken::Algorithm::HS256`.

### Token Generation

```rust
pub fn generate_token(&self, username: &str, role: &str) -> Result<String> {
    let now = chrono::Utc::now();
    let claims = Claims {
        sub: username.to_string(),
        jti: Uuid::new_v4().to_string(),
        iat: now.timestamp() as usize,
        exp: (now + chrono::Duration::hours(self.expiration_hours as i64)).timestamp() as usize,
        role: role.to_string(),
    };

    encode(&Header::default(), &claims, &self.encoding_key)
        .map_err(|e| AppError::Internal { message: format!("JWT encoding failed: {}", e) })
}
```

### Token Verification with Rotation

```rust
pub fn verify_token(&self, token: &str) -> Result<Claims> {
    // Try primary key first
    match decode::<Claims>(token, &self.decoding_key, &Validation::default()) {
        Ok(data) => {
            if self.is_blacklisted(&data.claims.jti) {
                return Err(AppError::Unauthorized { message: "Token revoked".into() });
            }
            return Ok(data.claims);
        }
        Err(_) => {
            // Try secondary key (rotation support)
            if let Some(ref secondary_key) = self.secondary_decoding_key {
                let data = decode::<Claims>(token, secondary_key, &Validation::default())
                    .map_err(|_| AppError::Unauthorized { message: "Invalid token".into() })?;
                if self.is_blacklisted(&data.claims.jti) {
                    return Err(AppError::Unauthorized { message: "Token revoked".into() });
                }
                return Ok(data.claims);
            }
        }
    }
    Err(AppError::Unauthorized { message: "Invalid token".into() })
}
```

---

## 14b. Auth Middleware (`src/auth/middleware.rs`)

```rust
use actix_web::{dev::ServiceRequest, Error, HttpMessage};
use actix_web::web::Data;
use std::sync::Arc;

/// actix-web middleware that validates JWT on protected routes.
/// Extracts Bearer token from Authorization header.
/// Sets the Claims as request extension for handlers to access.
pub async fn jwt_validator(
    req: ServiceRequest,
    credentials: Option<Data<Arc<JwtAuth>>>,
) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    // If auth is disabled, pass through
    let auth = match credentials {
        Some(a) => a,
        None => return Ok(req), // Auth disabled
    };

    // Extract Authorization header
    let auth_header = req.headers().get("Authorization")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| {
            // Return 401 Unauthorized
            let err = AppError::Unauthorized { message: "Missing Authorization header".into() };
            (err.error_response().into(), req)
        })?;

    // Must be "Bearer <token>"
    let token = auth_header.strip_prefix("Bearer ")
        .ok_or_else(|| {
            let err = AppError::Unauthorized { message: "Invalid Authorization format".into() };
            (err.error_response().into(), req)
        })?;

    // Verify token
    match auth.verify_token(token) {
        Ok(claims) => {
            req.extensions_mut().insert(claims);
            Ok(req)
        }
        Err(_) => {
            let err = AppError::Unauthorized { message: "Invalid or expired token".into() };
            Err((err.error_response().into(), req))
        }
    }
}

/// Extract Claims from request extensions (for use in handlers).
pub fn extract_claims(req: &HttpRequest) -> Result<Claims, AppError> {
    req.extensions()
        .get::<Claims>()
        .cloned()
        .ok_or_else(|| AppError::Unauthorized { message: "Not authenticated".into() })
}

/// Check if user has required role.
pub fn require_role(req: &HttpRequest, required_role: &str) -> Result<(), AppError> {
    let claims = extract_claims(req)?;
    if claims.role != required_role && claims.role != "admin" {
        return Err(AppError::Unauthorized { message: "Insufficient permissions".into() });
    }
    Ok(())
}
```

---

## 14c. Auth Handlers (`src/auth/handlers.rs`)

```rust
use actix_web::{web, HttpResponse};
use serde::Deserialize;

/// POST /auth/login
pub async fn login_handler(
    pool: web::Data<AnyPool>,
    auth: web::Data<Arc<JwtAuth>>,
    body: web::Json<LoginRequest>,
) -> HttpResponse;

/// POST /auth/refresh
pub async fn refresh_handler(
    auth: web::Data<Arc<JwtAuth>>,
    claims: Claims, // from middleware
) -> HttpResponse;

/// POST /auth/password
pub async fn change_password_handler(
    pool: web::Data<AnyPool>,
    auth: web::Data<Arc<JwtAuth>>,
    claims: Claims,
    body: web::Json<ChangePasswordRequest>,
) -> HttpResponse;

#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct ChangePasswordRequest {
    pub old_password: String,
    pub new_password: String,
}
```

### Login Handler Logic

```
login_handler:
  1. Rate limit check (5 req/min per IP, from governor)
  2. Look up user by username in DB
     └─ Not found or disabled → return 401 "Invalid credentials"
  3. Check if account is locked (locked_until > now)
     └─ Locked → return 401 "Account locked"
  4. bcrypt verify password against password_hash
     └─ Mismatch → increment failed_attempts, check lock threshold (5),
        return 401 "Invalid credentials"
  5. Check must_change_password
     └─ true → generate token, return 200 with must_change_password: true
  6. Reset failed_attempts, set last_login_at
  7. Generate JWT token
  8. Record audit log
  9. Return { token, token_type: "Bearer", expires_in }
```

### Password Change Handler Logic

```
change_password_handler:
  1. Validate old_password with bcrypt (prevents stolen-token password change)
     └─ Mismatch → 400 INVALID_OLD_PASSWORD
  2. Validate new_password: len ≥ 12, zxcvbn score, not in recent 5
     └─ Fails → 400 WEAK_PASSWORD with reason
  3. bcrypt hash new_password
  4. Update password_hash in DB, set must_change_password=false
  5. Blacklist all other tokens for this user (by jti)
  6. Record audit log
  7. Return success
```

### Bootstrap Admin

On first server startup, if `users` table is empty and `auth.enabled = true`:

```rust
pub async fn bootstrap_admin(pool: &AnyPool, config: &AuthConfig) -> Result<(), AppError> {
    let user_repo = UserRepository::new(pool.clone());
    let count = user_repo.count().await?;

    if count > 0 {
        return Ok(()); // Users already exist
    }

    // Read from environment variables
    let admin_user = std::env::var("CU_SCANNER_ADMIN_USER").ok();
    let admin_password = std::env::var("CU_SCANNER_ADMIN_PASSWORD").ok();

    match (admin_user, admin_password) {
        (Some(user), Some(pass)) => {
            let hash = bcrypt::hash(&pass, 12).map_err(...)?;
            user_repo.insert(&InsertUser {
                username: user,
                password_hash: hash,
                role: "admin".into(),
                status: Some("active".into()),
                must_change_password: Some(true), // Force change on first login
            }).await?;
            tracing::info!("Bootstrap admin user created: {}", user);
            Ok(())
        }
        _ => {
            tracing::error!(
                "Auth enabled but no users exist. Set CU_SCANNER_ADMIN_USER and \
                 CU_SCANNER_ADMIN_PASSWORD environment variables to bootstrap an admin."
            );
            Err(AppError::Config { message: "No users and no bootstrap credentials".into() })
        }
    }
}
```

## Test Cases

1. **Token generation**: Generate token → decode and verify username, role
2. **Token verification**: Valid token → returns Claims
3. **Expired token**: Token with past exp → verification fails
4. **Wrong secret**: Token signed with different key → verification fails
5. **Blacklist**: Blacklisted jti → verification fails
6. **Login success**: Correct credentials → returns token
7. **Login failure**: Wrong password → returns 401 (same message as user not found)
8. **Account locked**: 5 failed attempts → locked, login returns 401
9. **Password change**: Correct old password → password updated, must_change_password=false
10. **Weak password**: New password < 12 chars → rejected
11. **Secret rotation**: Token signed with old (secondary) key → still verifies
12. **Bootstrap admin**: Empty users table → admin created from env vars

## Acceptance Criteria

- [ ] JWT token includes sub, jti, iat, exp, role claims
- [ ] Token expiration is enforced
- [ ] Blacklist prevents revoked tokens from being used
- [ ] Login handler returns 401 for all failure types (no username enumeration)
- [ ] Password change requires old password verification
- [ ] 5 consecutive failures lock account for 15 minutes
- [ ] Password policy: ≥12 chars, zxcvbn check
- [ ] Bootstrap admin works on first startup
- [ ] Secret rotation (primary/secondary) works without service interruption
- [ ] All auth operations are audit-logged
