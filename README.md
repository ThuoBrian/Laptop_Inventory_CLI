
# Laptop Inventory CLI

A small Rust command-line application to manage a laptop inventory (users and laptops).

## Quick start

### Prerequisites

- Rust toolchain (stable) installed
- SQLite (or another DB) for persistent storage

### Build

```bash
cargo build
```

### Run (development)

```bash
cargo run -- <command> [args]
```

Migrations are stored in the `migrations/` folder — apply them to your database before using persistent storage.

## Common commands

- Create, list, update, and delete users and laptops via the CLI handlers (see `src/handlers/`).

## Testing & linting

- Run tests:

```bash
cargo test
```

- Lint & format:

```bash
cargo clippy --all-targets --all-features
cargo fmt --all
```

## Project structure

- `src/` — application source (handlers, models, db layer)
- `migrations/` — SQL migration files
- `Cargo.toml` — Rust manifest

## Contributing

Open an issue or submit a PR. Please run tests and linters before submitting changes.

## License

MIT (or specify your preferred license)
