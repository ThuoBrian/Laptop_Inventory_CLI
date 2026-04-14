# Laptop Inventory CLI

A REST API for managing laptop inventory and user assignments, built with Rust, Actix-web, and PostgreSQL.

## Tech Stack

| Layer      | Technology                          |
|------------|-------------------------------------|
| Language   | Rust (stable)                       |
| Web        | Actix-web 4                         |
| Database   | PostgreSQL via SQLx 0.8             |
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
```

### 2. Create the database & run migrations

```bash
psql -U <user> -c "CREATE DATABASE <dbname>;"
psql -U <user> -d <dbname> -f migrations/20250102_create_users.sql
psql -U <user> -d <dbname> -f migrations/20250101_create_laptops.sql
```

### 3. Build & run

```bash
cargo build
cargo run
```

Server starts at `http://127.0.0.1:5342`.

## API Endpoints

### Users

| Method   | Endpoint        | Description              | Body fields                          |
|----------|-----------------|--------------------------|--------------------------------------|
| `POST`   | `/users`        | Create a user            | `username`, `email`, `department`    |
| `GET`    | `/users`        | List all users           | —                                    |
| `GET`    | `/users/{id}`   | Get user by UUID         | —                                    |
| `PUT`    | `/users/{id}`   | Update user (partial OK) | `username?`, `email?`, `department?` |
| `DELETE` | `/users/{id}`   | Delete user              | —                                    |

> Deleting a user automatically unassigns all laptops assigned to them.

### Laptops

| Method   | Endpoint                  | Description                        | Body fields                                          |
|----------|---------------------------|------------------------------------|------------------------------------------------------|
| `POST`   | `/laptops`                | Add a laptop                       | `brand`, `model`, `serial_number`, `purchase_date`   |
| `GET`    | `/laptops`                | List all laptops                   | —                                                    |
| `GET`    | `/laptops?status=<value>` | Filter by status                   | —                                                    |
| `GET`    | `/laptops/{id}`           | Get laptop by UUID                 | —                                                    |
| `PUT`    | `/laptops/{id}`           | Update laptop (partial OK)         | `brand?`, `model?`, `serial_number?`, `status?`, `purchase_date?` |
| `DELETE` | `/laptops/{id}`           | Delete laptop                      | —                                                    |
| `POST`   | `/laptops/{id}/assign`    | Assign laptop to a user            | `user_id`                                            |
| `POST`   | `/laptops/{id}/unassign`  | Unassign laptop (→ available)      | —                                                    |

**Laptop status values:** `available` | `assigned` | `in_repair` | `retired`

> Use `/assign` and `/unassign` to manage the `assigned` status.  
> Setting `status` directly via `PUT` allows `available`, `in_repair`, and `retired` only.

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

# Filter available laptops
curl -s http://127.0.0.1:5342/laptops?status=available
```

## Project Structure

```
.
├── .cargo/
│   └── config.toml          # Linker configuration
├── migrations/
│   ├── 20250102_create_users.sql
│   └── 20250101_create_laptops.sql
├── src/
│   ├── main.rs              # Server setup & route registration
│   ├── models.rs            # Request/response data structures
│   ├── error.rs             # AppError enum + HTTP response mapping
│   ├── db/
│   │   ├── mod.rs
│   │   ├── users.rs         # User database operations
│   │   └── laptops.rs       # Laptop database operations
│   └── handlers/
│       ├── mod.rs
│       ├── users.rs         # User HTTP handlers
│       └── laptops.rs       # Laptop HTTP handlers
├── .env                     # Local secrets (git-ignored)
├── .env.example             # Template for .env
├── Cargo.toml
└── Cargo.lock
```

## Server Scripts

Convenience scripts for running the server in the background (Git Bash / WSL).

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

All errors return JSON:

```json
{ "error": "Laptop <id> cannot be assigned — current status is 'in_repair'." }
```

| Status | Meaning                        |
|--------|--------------------------------|
| 400    | Bad request / validation error |
| 404    | Resource not found             |
| 409    | Unique constraint violation    |
| 500    | Internal server error          |

## License

MIT
