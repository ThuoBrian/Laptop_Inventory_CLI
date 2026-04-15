# CLAUDE.md

## Project Overview

Laptop Inventory CLI — a Rust REST API + web UI for managing laptop inventory and user assignments. Built with Actix-web 4, SQLx 0.8 (PostgreSQL), Minijinja 2 templates, and HTMX for dynamic HTML.

## Architecture

```
Request → Actix-web Router → Handler → Validation → DB Layer (sqlx) → PostgreSQL
                                ↓
                          API handlers return JSON (/users, /laptops)
                          UI handlers return HTML (/ui, /ui/...)
```

- **API handlers** (`src/handlers/`): Return `HttpResponse::Ok().json(...)`. Accept `web::Json<T>`.
- **UI handlers** (`src/ui/`): Return rendered Minijinja HTML. Accept `web::Form<T>` for POST.
- **DB layer** (`src/db/`): Async functions taking `&PgPool`. No ORM — raw SQL via `sqlx::query_as`.
- **Models** (`src/models.rs`): `Laptop`, `User`, `LaptopStatus` enum, `PaginatedResponse<T>`, `LaptopWithAssignee`, request/response structs.
- **Validation** (`src/validation.rs`): `validate_required_string`, `validate_optional_string`, `validate_email`.
- **Templates** (`templates/`): Minijinja `.html` files. Base layout, pages, and HTMX partials.
- **Static** (`static/style.css`): Single CSS file, no framework.

## Key Patterns

- **LaptopStatus** is a Rust enum with `Display`/`FromStr`/`Serialize`/`Deserialize`/`sqlx::Type` impls. Status values in code use the enum, not strings.
- **Pagination**: All list endpoints accept `?page=&per_page=`. Defaults: page=1, per_page=50, max=100. Returns `PaginatedResponse<T>` with `data`, `total`, `page`, `per_page`, `total_pages`.
- **Validation**: Happens in handlers before calling DB. Required fields use `validate_required_string`, optional fields use `validate_optional_string`, email uses `validate_email`.
- **Transactions**: `delete_user` uses `pool.begin()` / `tx.commit()` to make the unassign + delete atomic.
- **Foreign key**: `laptops.assigned_to` references `users(id)` with `ON DELETE SET NULL`.
- **Error handling**: `AppError` enum with `NotFound`, `BadRequest`, `Conflict`, `Database` variants. `Database` returns generic "Internal server error" to clients and logs the real error. `Conflict` still leaks DB details (see priority_fixes.md).
- **Migrations**: Run automatically on startup via `sqlx::migrate!()`.
- **HTMX**: UI forms POST to `/ui/...` routes. Responses are HTML fragments that HTMX swaps into the DOM. `HX-Trigger: refreshTable` on mutation responses (note: currently causes double-fetch — see priority_fixes.md).

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
- HTML forms send empty strings for `Option<LaptopStatus>` and `Option<String>`, causing 400 errors instead of "no change"
- `HX-Trigger: refreshTable` causes a double-fetch on every form submission
- UI fragment handlers return JSON on error instead of HTML
- `Conflict` error variant leaks DB constraint details