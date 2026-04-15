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

### Add pagination to list endpoints
**Files:** `src/db/laptops.rs`, `src/db/users.rs`, `src/handlers/laptops.rs`, `src/handlers/users.rs`, `src/models.rs`
**Problem:** `get_all_laptops` and `get_all_users` return every row. With thousands of records these endpoints become slow and memory-heavy.
**Fix:** Add `limit`/`offset` query parameters with sensible defaults (e.g. limit=50). Return a paginated response struct with `{ data, total, page, per_page }`.

### Fix start.sh hardcoded address in output
**File:** `scripts/start.sh`
**Problem:** Line 76 prints `http://127.0.0.1:5342` even though the server now reads `HOST`/`PORT` from env vars. The printed address may be wrong.
**Fix:** Read `HOST` and `PORT` from `.env` in the script and print the actual address.

### Add connection pool configuration
**Files:** `src/main.rs`, `.env.example`
**Problem:** `PgPool::connect` uses default pool size (number of CPU cores). No way to tune for production load.
**Fix:** Use `PgPoolOptions::new().max_connections(N).connect_lazy()`. Read `DB_MAX_CONNECTIONS` from env with a default.

---

## Low

### Replace status strings with a Rust enum
**Files:** `src/models.rs`, `src/db/laptops.rs`, `src/handlers/laptops.rs`
**Problem:** `Laptop.status` is a `String`. Valid values are enforced at runtime via `validate_status` and a DB `CHECK` constraint. A typo in any string literal is a runtime bug, not a compile-time error.
**Fix:** Define `enum LaptopStatus { Available, Assigned, InRepair, Retired }` with `Display`/`FromStr` impls. Use it in the `Laptop` struct and eliminate `validate_status`.

### Fix email validation edge case
**File:** `src/validation.rs`
**Problem:** `validate_email` accepts `user@.` as valid — the domain part has a dot but no actual domain on each side.
**Fix:** Ensure the domain part has at least one character before and after the dot, e.g. check that the domain splits into at least two non-empty segments by `.`.

### Add request tracing / correlation IDs
**Files:** `src/main.rs` (new middleware)
**Problem:** No way to correlate a single request across multiple log lines or DB calls. Debugging issues in production is difficult.
**Fix:** Add a middleware that generates a UUID per request and includes it in all log output. Use actix-web's `middleware::DefaultHeaders` or a custom middleware to add `X-Request-Id` to responses.

### Add body size limits
**File:** `src/main.rs`
**Problem:** Actix-web accepts large JSON payloads by default. A malicious or buggy client could send a very large body.
**Fix:** Add `web::JsonConfig::default().limit(N)` as app data to cap JSON body size (e.g. 1MB).

### Add `updated_at` PostgreSQL trigger
**File:** New migration `migrations/20250104_add_updated_at_trigger.sql`
**Problem:** `updated_at` columns rely on `SET updated_at = NOW()` in every UPDATE query. If a raw SQL update forgets this, the timestamp goes stale.
**Fix:** Add a trigger function that auto-sets `updated_at = NOW()` on every row update for both `laptops` and `users` tables.

### Eliminate status magic strings
**Files:** `src/db/laptops.rs`, `migrations/20250101_create_laptops.sql`
**Problem:** Status values (`"available"`, `"assigned"`, `"in_repair"`, `"retired"`) appear as string literals in both Rust code and SQL. A name change requires updating multiple places.
**Fix:** Once the `LaptopStatus` enum is in place, derive the string values from the enum's `Display` impl. The SQL `CHECK` constraint is the single source of truth at the DB level.