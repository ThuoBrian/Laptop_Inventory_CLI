# Priority Fixes

Issues identified during staff engineer review, ordered by priority.

---

## High

### [DONE] Sanitize database errors before returning to clients
**File:** `src/error.rs`
**Problem:** `AppError::Database` passes raw `sqlx::Error` strings to clients, leaking table names, column names, and connection details.
**Fix:** Return generic "Internal server error" to clients; log the real error server-side.

---

## Medium

### [DONE] Add pagination to list endpoints
**Files:** `src/models.rs`, `src/db/laptops.rs`, `src/db/users.rs`, `src/handlers/laptops.rs`, `src/handlers/users.rs`
**Problem:** `get_all_laptops` and `get_all_users` return every row. With thousands of records these endpoints become slow and memory-heavy.
**Fix:** Added `limit`/`offset` query parameters with `PaginatedResponse` struct `{ data, total, page, per_page, total_pages }`. Defaults: page=1, per_page=50, max per_page=100.

### [DONE] Fix start.sh hardcoded address in output
**File:** `scripts/start.sh`
**Problem:** Line 76 prints `http://127.0.0.1:5342` even though the server now reads `HOST`/`PORT` from env vars. The printed address may be wrong.
**Fix:** Read `HOST` and `PORT` from env (with defaults) and print the actual address.

### [DONE] Add connection pool configuration
**Files:** `src/main.rs`, `.env.example`
**Problem:** `PgPool::connect` uses default pool size (number of CPU cores). No way to tune for production load.
**Fix:** Use `PgPoolOptions::new().max_connections(N).connect()`. Read `DB_MAX_CONNECTIONS` from env with default 10.

---

## Low

### [DONE] Replace status strings with a Rust enum
**Files:** `src/models.rs`, `src/db/laptops.rs`, `src/handlers/laptops.rs`
**Problem:** `Laptop.status` is a `String`. Valid values are enforced at runtime via `validate_status` and a DB `CHECK` constraint. A typo in any string literal is a runtime bug, not a compile-time error.
**Fix:** Defined `enum LaptopStatus { Available, Assigned, InRepair, Retired }` with `Display`/`FromStr`/`Serialize`/`Deserialize`/`sqlx::Type` impls. Used it in the `Laptop` struct. Eliminated `validate_status` function.

### [DONE] Fix email validation edge case
**File:** `src/validation.rs`
**Problem:** `validate_email` accepts `user@.` as valid — the domain part has a dot but no actual domain on each side.
**Fix:** Reject domains that are empty, start/end with a dot, or have no dot at all.

### [DONE] Add request tracing / correlation IDs
**File:** `src/request_id.rs`, `src/main.rs`
**Problem:** No way to correlate a single request across multiple log lines or DB calls. Debugging issues in production is difficult.
**Fix:** Added `RequestId` middleware that generates a UUID per request, logs `[request_id] METHOD /path`, and adds `X-Request-Id` header to responses.

### [DONE] Add body size limits
**File:** `src/main.rs`
**Problem:** Actix-web accepts large JSON payloads by default. A malicious or buggy client could send a very large body.
**Fix:** Added `web::JsonConfig::default().limit(1MB)` as app data.

### [DONE] Add `updated_at` PostgreSQL trigger
**File:** `migrations/20250104_add_updated_at_trigger.sql`
**Problem:** `updated_at` columns rely on `SET updated_at = NOW()` in every UPDATE query. If a raw SQL update forgets this, the timestamp goes stale.
**Fix:** Added `set_updated_at()` trigger function and triggers on both `laptops` and `users` tables.

### [DONE] Eliminate status magic strings
**Files:** `src/db/laptops.rs`, `src/models.rs`
**Problem:** Status values (`"available"`, `"assigned"`, `"in_repair"`, `"retired"`) appear as string literals in both Rust code and SQL. A name change requires updating multiple places.
**Fix:** Resolved by the `LaptopStatus` enum — string values are derived from `Display` impl. The SQL `CHECK` constraint remains the single source of truth at the DB level.