# UI Roadmap — Laptop Inventory CLI

## Overview

The backend REST API is complete. This roadmap covers building a UI on top of it so the
inventory can be managed without writing `curl` commands. Two UI layers are planned,
delivered in sequence:

| Phase | What | Why first |
|-------|------|-----------|
| 1 | Terminal UI (TUI) using **Ratatui** | Stays in Rust, no extra server, fast to build, fits the CLI identity |
| 2 | Web UI (HTML + HTMX) served by Actix-web | Browser-friendly, shareable, no framework overhead |

Each phase is self-contained. Phase 1 can be shipped and used before Phase 2 starts.

---

## Current State (Baseline)

```
REST API — Actix-web 4 + PostgreSQL
  POST/GET/PUT/DELETE  /users
  POST/GET/PUT/DELETE  /laptops
  POST                 /laptops/{id}/assign
  POST                 /laptops/{id}/unassign

Scripts
  scripts/start.sh | stop.sh | restart.sh
```

Interaction today requires `curl` or an API client (Postman, Bruno, etc.).

---

## Phase 1 — Terminal UI (Ratatui)

### Goal
A keyboard-driven terminal dashboard that lets you browse, create, edit, and assign
laptops and users without leaving the terminal.

### New dependencies

```toml
ratatui   = "0.29"     # TUI rendering
crossterm = "0.28"     # Cross-platform terminal control
```

### Screen layout

```
┌─ Laptop Inventory ──────────────────────────────────────── [q] Quit ─┐
│  [1] Laptops   [2] Users   [a] Assign   [n] New   [e] Edit   [d] Del │
├───────────────────────────────────────────────────────────────────────┤
│  Brand      Model           Serial         Status      Assigned To    │
│  ─────────  ───────────────  ─────────────  ──────────  ─────────────  │
│  Dell       Latitude 5540   SN-001         available   —              │
│▶ Lenovo     ThinkPad X1     SN-002         assigned    John Doe       │
│  HP         EliteBook 840   SN-003         in_repair   —              │
├───────────────────────────────────────────────────────────────────────┤
│  ↑↓ navigate   Enter select   Tab switch panel   ? help               │
└───────────────────────────────────────────────────────────────────────┘
```

### Key bindings

| Key | Action |
|-----|--------|
| `1` / `2` | Switch between Laptops / Users panel |
| `↑` `↓` | Navigate rows |
| `n` | Open "New" form |
| `e` | Open "Edit" form for selected row |
| `d` | Delete selected row (confirm prompt) |
| `a` | Assign selected laptop → pick user from list |
| `u` | Unassign selected laptop |
| `s` | Filter by status |
| `/` | Search by brand, model, or serial |
| `?` | Help overlay |
| `q` | Quit |

### New source files

```
src/
  tui/
    mod.rs          — app entry point, event loop
    app.rs          — shared application state
    ui.rs           — Ratatui widget rendering
    events.rs       — keyboard input handling
    forms.rs        — new/edit input form widgets
```

### Milestones

- [ ] **1.1** Bootstrap Ratatui event loop, render empty table
- [ ] **1.2** Laptops panel — list view with live data from DB
- [ ] **1.3** Users panel — list view with live data from DB
- [ ] **1.4** Create form — add new laptop / user
- [ ] **1.5** Edit form — update existing record
- [ ] **1.6** Delete with confirmation prompt
- [ ] **1.7** Assign / Unassign flow (pick user from popup list)
- [ ] **1.8** Status filter + keyword search
- [ ] **1.9** Help overlay, error toast messages
- [ ] **1.10** `--tui` flag in `main.rs` to launch TUI instead of the HTTP server

### How to run (planned)

```bash
cargo run -- --tui        # launch TUI (connects directly to DB)
cargo run                 # launch REST API (existing behaviour)
```

---

## Phase 2 — Web UI (HTML + HTMX)

### Goal
A lightweight browser-based interface served directly by the existing Actix-web server.
No separate frontend build step. No JavaScript framework.

### Approach
- **HTMX** handles dynamic updates (server returns HTML fragments, no JSON needed in the browser)
- **Actix-web** serves static files + new HTML template routes
- **Minijinja** (or `askama`) for server-side HTML templating in Rust

### New dependencies

```toml
actix-files = "0.6"      # serve static assets
minijinja   = "2"        # server-side HTML templates
```

### Routes added

| Method | Path | Returns |
|--------|------|---------|
| `GET` | `/ui` | Dashboard page (HTML) |
| `GET` | `/ui/laptops` | Laptops table fragment (HTMX target) |
| `GET` | `/ui/users` | Users table fragment (HTMX target) |
| `POST` | `/ui/laptops/new` | Create laptop, return updated table |
| `POST` | `/ui/users/new` | Create user, return updated table |
| `POST` | `/ui/laptops/{id}/assign` | Assign, return updated row |

### UI layout (browser)

```
┌─────────────────────────────────────────────────────────┐
│  Laptop Inventory                           Brian Thuo   │
├────────────┬────────────────────────────────────────────┤
│  Laptops   │  Brand    Model     Serial    Status   ...  │
│  Users     │  Dell     Lat 5540  SN-001    available     │
│            │  Lenovo   X1 Carbon SN-002    assigned      │
│  [+ New]   │                                             │
│            │                         [Assign] [Edit] [X] │
└────────────┴────────────────────────────────────────────┘
```

### New source files

```
src/
  web/
    mod.rs          — registers UI routes
    pages.rs        — full-page handlers (dashboard)
    fragments.rs    — HTMX partial handlers (table rows, forms)

static/
  style.css         — minimal stylesheet (no framework)

templates/
  base.html
  laptops.html
  users.html
  partials/
    laptop_row.html
    user_row.html
    laptop_form.html
    user_form.html
```

### Milestones

- [ ] **2.1** Serve `GET /ui` dashboard page with Actix-files + Minijinja
- [ ] **2.2** Laptops table — load via HTMX on page load
- [ ] **2.3** Users table — load via HTMX on page load
- [ ] **2.4** "New Laptop" slide-in form, submit and refresh table
- [ ] **2.5** "New User" slide-in form, submit and refresh table
- [ ] **2.6** Inline edit row
- [ ] **2.7** Delete with confirmation dialog
- [ ] **2.8** Assign modal — pick user from dropdown
- [ ] **2.9** Status badge colouring (`available` = green, `assigned` = blue, etc.)
- [ ] **2.10** Basic CSS polish — responsive layout, mobile-friendly table

---

## Decisions Log

| Decision | Rationale |
|----------|-----------|
| Ratatui over a desktop GUI (Tauri/egui) | Faster to build, zero install, Rust-native |
| HTMX over React/Vue | No build step, no npm, Rust does all rendering |
| Minijinja over Askama | Runtime templates — easier to iterate without recompiling |
| Phase 1 before Phase 2 | TUI works offline, no browser needed — better for field use |

---

## Out of Scope (for now)

- Authentication / login screen
- Multi-user access control
- Mobile app
- Real-time push notifications (WebSockets)
- Export to CSV / Excel (can be added as a quick win in Phase 2)

---

## Quick-start checklist before Phase 1

- [ ] Linker issue resolved (`cargo build` succeeds)
- [ ] Migrations applied to PostgreSQL
- [ ] `.env` configured and server starts cleanly
- [ ] At least one user and one laptop created via `curl` to test the DB layer
