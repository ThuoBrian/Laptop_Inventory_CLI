# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Laptop Inventory CLI — a Rust REST API + web UI for managing laptop inventory and user assignments. Built with Actix-web 4, SQLx 0.8 (PostgreSQL), Minijinja 2 templates, and HTMX for dynamic HTML.

## Architecture

```
Request → RequestId middleware → Actix-web Router → Handler → Validation → DB Layer (sqlx) → PostgreSQL
                                                        ↓
                                                  API handlers return JSON (/users, /laptops)
                                                  UI handlers return HTML (/ui, /ui/...)
```

- **API handlers** (`src/handlers/`): Return `HttpResponse::Ok().json(...)`. Accept `web::Json<T>`.
- **UI handlers** (`src/ui/`): Return rendered Minijinja HTML. Accept `web::Form<T>` for POST.
- **DB layer** (`src/db/`): Async functions taking `&PgPool`. No ORM — raw SQL via `sqlx::query_as`.
- **Models** (`src/models.rs`): `Laptop`, `User`, `LaptopStatus` enum, `PaginatedResponse<T>`, `LaptopWithAssignee`, request/response structs.
- **Validation** (`src/validation.rs`): `validate_required_string`, `validate_optional_string`, `validate_email`.
- **Templates** (`templates/`): Minijinja `.html` files loaded from disk at request time (not embedded). Changes to templates take effect on the next request without restarting the server.
- **Static** (`static/style.css`): Single CSS file, no framework.
- **RequestId middleware** (`src/request_id.rs`): Generates a UUID per request, logs `[uuid] METHOD PATH` at INFO, and injects it as the `X-Request-Id` response header.

## Key Patterns

- **LaptopStatus** is a Rust enum with `Display`/`FromStr`/`Serialize`/`Deserialize`/`sqlx::Type` impls. Status values in code use the enum, not strings.
- **Pagination**: All list endpoints accept `?page=&per_page=`. Defaults: page=1, per_page=50, max=100. Returns `PaginatedResponse<T>` with `data`, `total`, `page`, `per_page`, `total_pages`.
- **Validation**: Happens in handlers before calling DB. Required fields use `validate_required_string`, optional fields use `validate_optional_string`, email uses `validate_email`. All fields are capped at 100 characters. **Asymmetry**: `validate_optional_string(Some(""))` returns `Err(BadRequest)`, not `Ok(None)` — the serde `deserialize_optional_trimmed` helper converts empty → `None` before validation is called, so the two are designed to work in sequence.
- **Status transitions**: `assign_laptop` only succeeds when the laptop's current status is `available`; `unassign_laptop` only succeeds when it is `assigned`. Both enforce this in the UPDATE `WHERE` clause (not a separate read). Setting status to `assigned` via the update endpoint is rejected with `BadRequest`.
- **Transactions**: `delete_user` uses `pool.begin()` / `tx.commit()` to make the unassign + delete atomic.
- **Foreign key**: `laptops.assigned_to` references `users(id)` with `ON DELETE SET NULL`.
- **Error handling**: `AppError` enum with `NotFound`, `BadRequest`, `Conflict`, `Database` variants. `Database` returns a generic "Internal server error" (and logs the real error); `Conflict` returns a sanitized message looked up from the constraint name (raw PG detail is logged, not exposed). UI handlers return `UiResult` — a `Responder` newtype that renders `partials/error.html` via `AppError::to_html` on error, by pulling the Minijinja `Environment` out of `req.app_data`. API handlers continue to return `Result<_, AppError>` (JSON errors).
- **Form deserialization**: `UpdateLaptop` / `UpdateUser` optional fields use `#[serde(deserialize_with = ...)]` helpers in `src/models.rs` (`deserialize_optional_trimmed`, `deserialize_optional_status`, `deserialize_optional_date`) so empty/whitespace form values become `None` ("no change") instead of 400 errors.
- **DB optional-filter queries**: Filtered vs. unfiltered query variants are written as separate `match` arms (see `db::laptops::get_all_laptops` and `get_all_laptops_with_assignee`), not dynamic SQL. Adding a new filter requires updating both arms.
- **`LaptopWithAssignee`**: Used only by the UI layer (LEFT JOINs the `users` table for the assignee name). The JSON API uses plain `Laptop`.
- **Migrations**: Run automatically on startup via `sqlx::migrate!()`. Files in `migrations/` follow the `YYYYMMDD_description.sql` naming convention.
- **HTMX**: UI forms POST to `/ui/...` routes. Responses are HTML fragments that HTMX swaps into the DOM via each form's `hx-target` — no `HX-Trigger`-based refresh, tables rely on the direct swap.

## Commands

```bash
cargo build                    # Build
cargo run                      # Run (reads .env for DATABASE_URL, HOST, PORT)
cargo test                     # Run tests (none exist yet)
cargo clippy --all-targets     # Lint
cargo fmt --all                # Format
```

Server scripts: `./scripts/start.sh`, `./scripts/stop.sh`, `./scripts/restart.sh`

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `DATABASE_URL` | required | PostgreSQL connection string |
| `HOST` | `127.0.0.1` | Server bind address |
| `PORT` | `5342` | Server bind port |
| `RUST_LOG` | `info` | Log level |
| `DB_MAX_CONNECTIONS` | `10` | PgPool max connections |

## Route Summary

**API (JSON)**:
- `POST/GET/PUT/DELETE /users`, `POST/GET/PUT/DELETE /laptops`
- `POST /laptops/{id}/assign`, `POST /laptops/{id}/unassign`

**Web UI (HTML)**:
- `GET /ui` — Dashboard
- `GET /ui/laptops` — Laptops page (HTMX: table fragment)
- `GET /ui/users` — Users page (HTMX: table fragment)
- `POST /ui/laptops/new`, `POST /ui/laptops/{id}/edit`, `POST /ui/laptops/{id}/delete`
- `GET /ui/laptops/{id}/assign-form`, `POST /ui/laptops/{id}/assign`, `POST /ui/laptops/{id}/unassign`
- `GET /ui/laptops/{id}/edit-form`
- `POST /ui/users/new`, `GET /ui/users/{id}/edit-form`, `POST /ui/users/{id}/edit`, `POST /ui/users/{id}/delete`

## Known Issues

See `priority_fixes.md` for the full list. The most impactful open items:
- Modal closes on form error (HTMX `hx-on::after-request`), hiding the error banner the server now returns
- No CSRF protection on `/ui/...` POST routes
- Status filter is dropped when paginating the laptops table
- API handlers don't clamp `page`/`per_page` (UI handlers do)