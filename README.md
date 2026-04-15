# Laptop Inventory CLI

A REST API and web UI for managing laptop inventory and user assignments, built with Rust, Actix-web, and PostgreSQL.

## Tech Stack

| Layer      | Technology                          |
|------------|-------------------------------------|
| Language   | Rust (stable, edition 2021)        |
| Web        | Actix-web 4                         |
| Database   | PostgreSQL via SQLx 0.8             |
| Templates  | Minijinja 2 (server-side HTML)     |
| UI         | HTMX 2 (dynamic HTML over the wire)|
| Runtime    | Tokio (async)                       |
| Serialiser | Serde / serde_json                  |

## Prerequisites

- [Rust](https://rustup.rs/) stable toolchain
- PostgreSQL 13+ running locally (or remotely)

## Setup

### 1. Clone & configure environment

```bash
git clone https://github.com/ThuoBrian/Laptop_Inventory_CLI.git
cd Laptop_Inventory_CLI
cp .env.example .env        # then edit .env with your DB credentials
```

`.env` format:

```env
DATABASE_URL=postgres://<user>:<password>@localhost:5432/<dbname>
RUST_LOG=info
HOST=127.0.0.1
PORT=5342
DB_MAX_CONNECTIONS=10
```

### 2. Create the database

```bash
psql -U <user> -c "CREATE DATABASE <dbname>;"
```

### 3. Build & run

```bash
cargo build
cargo run
```

Migrations run automatically on startup. The server starts at `http://127.0.0.1:5342`.

## Web UI

Open `http://127.0.0.1:5342/ui` in your browser for a full management interface:

- **Dashboard** — summary cards with laptop counts by status
- **Laptops** — browse, create, edit, assign, unassign, and delete laptops
- **Users** — browse, create, edit, and delete users

The UI uses HTMX for dynamic updates (no page reloads, no JavaScript framework) and Minijinja for server-side HTML rendering. Status badges are color-coded: available (green), assigned (blue), in repair (amber), retired (gray).

## API Endpoints

### Users

| Method   | Endpoint        | Description              | Body fields                          |
|----------|-----------------|--------------------------|--------------------------------------|
| `POST`   | `/users`        | Create a user            | `username`, `email`, `department`    |
| `GET`    | `/users`        | List users (paginated)  | `?page=1&per_page=50`               |
| `GET`    | `/users/{id}`   | Get user by UUID         | —                                    |
| `PUT`    | `/users/{id}`   | Update user (partial OK) | `username?`, `email?`, `department?` |
| `DELETE` | `/users/{id}`   | Delete user              | —                                    |

> Deleting a user automatically unassigns all laptops assigned to them (atomic transaction).

### Laptops

| Method   | Endpoint                  | Description                        | Body fields                                          |
|----------|---------------------------|------------------------------------|------------------------------------------------------|
| `POST`   | `/laptops`                | Add a laptop                       | `brand`, `model`, `serial_number`, `purchase_date`   |
| `GET`    | `/laptops`                | List laptops (paginated)           | `?status=available&page=1&per_page=50`              |
| `GET`    | `/laptops/{id}`           | Get laptop by UUID                 | —                                                    |
| `PUT`    | `/laptops/{id}`           | Update laptop (partial OK)         | `brand?`, `model?`, `serial_number?`, `status?`, `purchase_date?` |
| `DELETE` | `/laptops/{id}`           | Delete laptop                      | —                                                    |
| `POST`   | `/laptops/{id}/assign`    | Assign laptop to a user            | `user_id`                                            |
| `POST`   | `/laptops/{id}/unassign`  | Unassign laptop (→ available)      | —                                                    |

**Laptop status values:** `available` | `assigned` | `in_repair` | `retired`

> Use `/assign` and `/unassign` to manage the `assigned` status.
> Setting `status` directly via `PUT` allows `available`, `in_repair`, and `retired` only.

### Input Validation

All string fields are validated:
- Non-empty (after trimming whitespace)
- Maximum 100 characters
- Email must contain `@` and a valid domain (e.g., `user@example.com`)

### Example requests

```bash
# Create a user
curl -s -X POST http://127.0.0.1:5342/users \
  -H "Content-Type: application/json" \
  -d '{"username":"jdoe","email":"jdoe@example.com","department":"IT"}'

# Add a laptop
curl -s -X POST http://127.0.0.1:5342/laptops \
  -H "Content-Type: application/json" \
  -d '{"brand":"Dell","model":"Latitude 5540","serial_number":"SN-001","purchase_date":"2024-01-15"}'

# Assign the laptop to the user (replace UUIDs)
curl -s -X POST http://127.0.0.1:5342/laptops/<laptop-id>/assign \
  -H "Content-Type: application/json" \
  -d '{"user_id":"<user-id>"}'

# List available laptops (page 1)
curl -s http://127.0.0.1:5342/laptops?status=available&page=1&per_page=10
```

## Project Structure

```
.
├── migrations/
│   ├── 20250101_create_laptops.sql
│   ├── 20250102_create_users.sql
│   ├── 20250103_add_fk_laptops_assigned_to.sql
│   └── 20250104_add_updated_at_trigger.sql
├── src/
│   ├── main.rs              # Server setup & route registration
│   ├── models.rs            # Data structures & LaptopStatus enum
│   ├── error.rs             # AppError enum + HTTP/HTML response mapping
│   ├── validation.rs        # Input validation helpers
│   ├── request_id.rs        # X-Request-Id middleware
│   ├── db/
│   │   ├── mod.rs
│   │   ├── users.rs         # User database operations
│   │   └── laptops.rs       # Laptop database operations
│   ├── handlers/
│   │   ├── mod.rs
│   │   ├── users.rs         # User API handlers
│   │   └── laptops.rs       # Laptop API handlers
│   └── ui/
│       ├── mod.rs
│       ├── templates.rs     # Minijinja environment setup
│       ├── pages.rs         # Full-page HTML handlers
│       └── fragments.rs     # HTMX partial handlers
├── templates/
│   ├── base.html
│   ├── pages/
│   │   ├── dashboard.html
│   │   ├── laptops.html
│   │   └── users.html
│   └── partials/
│       ├── laptop_table.html
│       ├── user_table.html
│       ├── laptop_form.html
│       ├── user_form.html
│       ├── assign_modal.html
│       └── error.html
├── static/
│   └── style.css
├── scripts/
│   ├── start.sh
│   ├── stop.sh
│   └── restart.sh
├── .env.example
├── Cargo.toml
└── Cargo.lock
```

## Server Scripts

Convenience scripts for running the server in the background.

```bash
# Make executable (first time only)
chmod +x scripts/*.sh

# Start server (release build)
./scripts/start.sh

# Start server (debug build — faster compile)
./scripts/start.sh --dev

# Stop server gracefully
./scripts/stop.sh

# Restart server
./scripts/restart.sh
```

Logs are written to `server.log`. Tail them live with:

```bash
tail -f server.log
```

## Development

```bash
# Run interactively with live logging (Ctrl+C to stop)
RUST_LOG=debug cargo run

# Lint
cargo clippy --all-targets --all-features

# Format
cargo fmt --all

# Tests
cargo test
```

## Error Responses

API errors return JSON:

```json
{ "error": "Laptop <id> cannot be assigned — current status is 'in_repair'." }
```

| Status | Meaning                        |
|--------|--------------------------------|
| 400    | Bad request / validation error |
| 404    | Resource not found             |
| 409    | Unique constraint violation    |
| 500    | Internal server error          |

> Database error details are logged server-side and never exposed to clients.

## License

MIT