# Priority Fixes

Issues identified during staff engineer and code review, ordered by priority.

---

## Critical (will cause errors or incorrect behavior)

### `Option<LaptopStatus>` deserialization fails on empty form value
**Files:** `src/models.rs:116`, `templates/partials/laptop_form.html:26`
**Problem:** The edit laptop form has `<option value="">-- No change --</option>`. When submitted, the browser sends `status=` (empty string). The `FromStr` impl on `LaptopStatus` rejects it, producing a 400 error instead of "no change" behavior.
**Fix:** Use a custom deserializer for `Option<LaptopStatus>` that maps empty strings to `None`, or handle it in the fragment handler before passing to the DB layer.

### `Option<String>` fields get `Some("")` from forms — `empty_to_none` is dead code
**Files:** `src/ui/fragments.rs:7-9`, `src/ui/fragments.rs:106-108`, `src/validation.rs:37-39`
**Problem:** HTML forms submit empty strings for unfilled fields, deserialized as `Some("")`. `validate_optional_string` rejects `Some("")` as "cannot be empty." The `empty_to_none` helper exists but is never called.
**Fix:** Apply `empty_to_none` to all `Option<String>` fields in fragment handlers before validation.

### Race condition in assign/unassign (TOCTOU)
**Files:** `src/db/laptops.rs:153-208`
**Problem:** `assign_laptop` and `unassign_laptop` read status with `get_laptop_by_id`, then issue an UPDATE. Between those queries, another request could change the status. Two simultaneous assigns could both succeed.
**Fix:** Move the status check into the UPDATE's WHERE clause: `WHERE id = $1 AND status = 'available'`. Check `rows_affected()` to determine success.

---

## High (broken UX or information leakage)

### `HX-Trigger: refreshTable` causes double-fetch on every form submission
**File:** `src/ui/fragments.rs:24`
**Problem:** Every fragment response includes `HX-Trigger: refreshTable`. The table divs listen for `refreshTable from:body`. After a form swaps the table HTML, the event fires and causes a second redundant GET request.
**Fix:** Remove `HX-Trigger` from fragment responses since the table is already being swapped directly by the form's `hx-target`.

### Modal closes on form error, losing user feedback
**Files:** `templates/partials/laptop_form.html:5`, `templates/partials/user_form.html:5`, `templates/partials/assign_modal.html:5`
**Problem:** `hx-on::after-request` fires regardless of success or failure (4xx/5xx). On validation error, the modal closes and the user sees no feedback.
**Fix:** Use `hx-on::after-settle` with a success check, or use `hx-on::response-error` to keep the modal open on errors. Alternatively, redirect errors to an in-modal error display.

### UI fragment handlers return JSON errors instead of HTML
**Files:** `src/ui/fragments.rs` (all handlers), `src/error.rs:64-82`
**Problem:** Fragment handlers return `Result<HttpResponse, AppError>`. On error, Actix calls `ResponseError::error_response()` which returns JSON. HTMX injects raw JSON into the DOM target. The `AppError::to_html()` method exists but is never called.
**Fix:** Wrap UI handlers with error-catching logic that calls `AppError::to_html()` instead of letting Actix use the default JSON response.

### `Conflict` error variant leaks database constraint details
**File:** `src/error.rs:52-53`
**Problem:** When a unique constraint is violated (error code 23505), the raw PostgreSQL message is returned to clients. This includes constraint name, table name, and column values (e.g., `duplicate key value violates unique constraint "users_email_key"`).
**Fix:** Return a generic conflict message (e.g., "A record with this value already exists") and log the full details server-side.

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

### No lower-bound validation on `page`/`per_page` parameters
**Files:** All handlers that accept `page`/`per_page` query params
**Problem:** `page=0` or `page=-1` produces a negative offset. `per_page=0` causes division by zero in `PaginatedResponse::new`.
**Fix:** Clamp `page` to `max(1, page)` and `per_page` to `max(1, per_page)`.

---

## Low

### Dead code: `empty_to_none` function
**File:** `src/ui/fragments.rs:7-9`
**Problem:** Defined but never called. See the critical bug above — this is the fix for the `Some("")` form issue.

### Duplicated `render_template`, `LaptopListQuery`, `PaginationParams`
**Files:** `src/ui/pages.rs`, `src/ui/fragments.rs`, `src/handlers/laptops.rs`, `src/handlers/users.rs`
**Problem:** Same types and helper functions defined in multiple places.
**Fix:** Move shared query structs and `render_template` to a common location (e.g., `src/models.rs` for query types, `src/ui/mod.rs` for the template helper).

### Assign modal capped at 100 users
**File:** `src/ui/fragments.rs:136`
**Problem:** `get_all_users(&pool, 1, MAX_PER_PAGE)` limits the dropdown to 100 users. If there are more, some are missing with no way to search.
**Fix:** Add a search-as-you-type endpoint, or increase the limit for dropdowns specifically.

### UUID oracle in NotFound error messages
**Files:** `src/db/laptops.rs:98`, `src/db/users.rs:61`
**Problem:** Error messages include the UUID, allowing enumeration of valid vs invalid IDs.
**Fix:** Return a generic "Resource not found" message without the UUID.

### No status filter UI on the laptops page
**File:** `templates/pages/laptops.html`
**Problem:** The backend supports `?status=available` filtering but there is no UI for it.
**Fix:** Add a status filter dropdown or button row above the table.

---

## Completed (from previous review)

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