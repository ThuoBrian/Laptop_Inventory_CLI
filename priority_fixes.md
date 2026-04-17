# Priority Fixes

Issues identified during staff engineer and code review, ordered by priority.

---

## Critical (will cause errors or incorrect behavior)

_All previously-Critical items are now resolved — see Completed section._

---

## High (broken UX or information leakage)

### Modal closes on form error, losing user feedback
**Files:** `templates/partials/laptop_form.html:5`, `templates/partials/user_form.html:5`, `templates/partials/assign_modal.html:5`
**Problem:** `hx-on::after-request` fires regardless of success or failure (4xx/5xx). On validation error, the modal closes and the user sees no feedback. The server now *does* return a rendered `partials/error.html` fragment on error (see the UI-fragment fix below), but HTMX still closes the modal so the user never sees it.
**Fix:** Use `hx-on::after-settle` with a success check, or use `hx-on::response-error` to keep the modal open on errors. Alternatively, redirect errors to an in-modal error display (e.g., `hx-target-error` pointed at an error container inside the modal).

---

## Medium

### No CSRF protection on form endpoints
**Files:** All `POST /ui/...` routes in `src/ui/fragments.rs`
**Problem:** No CSRF tokens on any form submissions. An attacker could craft a malicious page that auto-submits forms.
**Fix:** Add actix-csrf or a custom middleware that validates a CSRF token on all POST requests to `/ui/...` routes.

### Status filter lost on pagination
**File:** `templates/partials/laptop_table.html:63-75`
**Problem:** Pagination links are `hx-get="/ui/laptops?page={{ page - 1 }}"` with no `status` parameter. If a user filters by status and clicks Next, the filter is dropped.
**Fix:** Preserve the current status filter in pagination links by passing it as a template variable.

### No lower-bound validation on `page`/`per_page` in API handlers
**Files:** `src/handlers/laptops.rs:32-33`, `src/handlers/users.rs` (list handler)
**Problem:** `page=0` or `page=-1` produces a negative offset. `per_page=0` causes division by zero in `PaginatedResponse::new`. UI routes now clamp via `clamp_page`/`clamp_per_page` in `src/ui/pages.rs`, but the JSON API handlers still pass the raw values through.
**Fix:** Lift the `clamp_page`/`clamp_per_page` helpers to a shared location (e.g., `src/models.rs`) and apply them in the API handlers as well.

---

## Low

### Duplicated `render_template`, `LaptopListQuery`, `PaginationParams`
**Files:** `src/ui/pages.rs`, `src/ui/fragments.rs`, `src/handlers/laptops.rs`, `src/handlers/users.rs`
**Problem:** Same types and helper functions defined in multiple places.
**Fix:** Move shared query structs and `render_template` to a common location (e.g., `src/models.rs` for query types, `src/ui/mod.rs` for the template helper).

### Assign modal capped at 100 users
**File:** `src/ui/fragments.rs` (see call to `get_all_users(&pool, 1, MAX_PER_PAGE)`)
**Problem:** `MAX_PER_PAGE` limits the dropdown to 100 users. If there are more, some are missing with no way to search.
**Fix:** Add a search-as-you-type endpoint, or increase the limit for dropdowns specifically.

### UUID oracle in NotFound error messages
**Files:** `src/db/laptops.rs`, `src/db/users.rs`
**Problem:** Error messages include the UUID, allowing enumeration of valid vs invalid IDs.
**Fix:** Return a generic "Resource not found" message without the UUID.

### No status filter UI on the laptops page
**File:** `templates/pages/laptops.html`
**Problem:** The backend supports `?status=available` filtering but there is no UI for it.
**Fix:** Add a status filter dropdown or button row above the table.

---

## Completed

### [DONE] `Option<LaptopStatus>` and `Option<String>` empty-form-value deserialization
**Files:** `src/models.rs`, `src/ui/fragments.rs`
**Resolution:** Added `deserialize_optional_trimmed`, `deserialize_optional_status`, and `deserialize_optional_date` helpers in `src/models.rs` and applied them via `#[serde(deserialize_with = ...)]` on every optional field of `UpdateLaptop` and `UpdateUser`. Empty/whitespace form values now deserialize to `None` (the "no change" semantic) instead of producing a 400. Subsumes the old "`empty_to_none` is dead code" entry — the helper has been removed from `fragments.rs`.

### [DONE] Race condition in assign/unassign (TOCTOU)
**File:** `src/db/laptops.rs`
**Resolution:** Moved the status check into the UPDATE's WHERE clause (`WHERE id = $1 AND status = 'available'` / `'assigned'`). Uses `fetch_optional`; on `None`, re-reads the laptop to produce an accurate `NotFound` vs. `BadRequest` error. Two concurrent assigns can no longer both succeed.

### [DONE] `HX-Trigger: refreshTable` double-fetch
**Files:** `src/ui/fragments.rs`, `templates/pages/laptops.html`, `templates/pages/users.html`
**Resolution:** Dropped the `HX-Trigger: refreshTable` header from `render_template` and removed the now-unused `refreshTable from:body` listener from the table divs. Forms swap the table directly via `hx-target`.

### [DONE] UI fragment handlers return JSON errors instead of HTML
**Files:** `src/error.rs`, `src/ui/fragments.rs`, `src/ui/pages.rs`
**Resolution:** Added `UiResult` — a `Responder` newtype over `Result<HttpResponse, AppError>`. On `Err`, it looks up the Minijinja `Environment` via `req.app_data::<web::Data<Environment<'static>>>()` and renders `partials/error.html` via `AppError::to_html`. Both UI fragment handlers and UI page handlers now return `UiResult`, so HTMX swaps a rendered error banner into the DOM instead of raw JSON. API handlers continue to use `Result<_, AppError>` (JSON errors, correct for JSON clients).

### [DONE] `Conflict` error variant leaks database constraint details
**File:** `src/error.rs`
**Resolution:** On `23505` (unique violation), we now log the raw constraint name and PG message server-side via `log::warn!`, and return a user-facing message looked up from a small table of known constraints (`users_email_key` → "A user with this email already exists.", etc.) with a generic fallback.

### [DONE] `page`/`per_page` lower-bound clamping (UI handlers)
**File:** `src/ui/pages.rs`
**Resolution:** Added `clamp_page` (`max(1, page)`) and `clamp_per_page` (`clamp(1, MAX_PER_PAGE)`) helpers and applied them to `laptops_page` and `users_page`. API handlers still need the same treatment — see the Medium entry above.

---

## Completed (from earlier reviews)

### [DONE] Sanitize database errors before returning to clients
**File:** `src/error.rs`

### [DONE] Add pagination to list endpoints
**Files:** `src/models.rs`, `src/db/laptops.rs`, `src/db/users.rs`, `src/handlers/laptops.rs`, `src/handlers/users.rs`

### [DONE] Fix start.sh hardcoded address in output
**File:** `scripts/start.sh`

### [DONE] Add connection pool configuration
**Files:** `src/main.rs`, `.env.example`

### [DONE] Replace status strings with a Rust enum
**Files:** `src/models.rs`, `src/db/laptops.rs`, `src/handlers/laptops.rs`

### [DONE] Fix email validation edge case
**File:** `src/validation.rs`

### [DONE] Add request tracing / correlation IDs
**File:** `src/request_id.rs`, `src/main.rs`

### [DONE] Add body size limits
**File:** `src/main.rs`

### [DONE] Add `updated_at` PostgreSQL trigger
**File:** `migrations/20250104_add_updated_at_trigger.sql`

### [DONE] Eliminate status magic strings
**Files:** `src/db/laptops.rs`, `src/models.rs`
