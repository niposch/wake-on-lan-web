# 🦀 WoL Backend

The core logic of the application. Written in Rust using Axum.

## Features
* **API:** REST endpoints to manage devices and users.
* **Static Serving:** Serves the compiled React frontend.
* **WoL:** Broadcasts Magic Packets via UDP.
* **Ping:** Checks device status via ICMP (or shell execution).

## Development

### Environment Variables
Create a `.env` file in this directory:
```env
DATABASE_URL=sqlite:wol.db
RUST_LOG=debug

```

### Database Management

We use `sqlx` for compile-time verified queries.

```bash
# Create a new migration file
sqlx migrate add <name_of_change>

# Apply migrations
sqlx migrate run

# Revert last migration
sqlx migrate revert

```

### Key Dependencies

* **Axum:** Web framework.
* **SQLx:** Async SQLite driver.
* **Wake-on-lan:** Magic packet construction.
* **Tokio:** Async runtime.
