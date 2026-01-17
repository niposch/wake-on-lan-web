# ⚡ Wake-on-LAN Web

A lightweight, self-hosted web interface to wake up and manage devices on your local network.

## 🏗 Architecture

This is a monolithic repository containing three distinct parts:

* **Frontend:** A React Single Page Application (SPA) built with Vite and Bun.
* **Backend:** A Rust (Axum) API that serves the frontend, manages the SQLite database, and handles networking (WoL packets / ICMP pings).
* **Agent (Optional):** A lightweight background service running on target PCs to handle remote shutdown.

## 🚀 Quick Start

### Prerequisites
* **Rust:** `1.75+`
* **Bun:** `1.0+`
* **Node.js:** (Required for some Vite internal tooling)

### 1. Setup Backend
```bash
cd backend
# Create the .env file
echo "DATABASE_URL=sqlite:wol.db" > .env
# Initialize Database & Migrations
sqlx database create
sqlx migrate run
# Run the server
cargo run

```

### 2. Setup Frontend

```bash
cd frontend
# Install dependencies
bun install
# Start Dev Server (with proxy to backend)
bun dev

```

### 3. Production Build

To run the app as a single binary:

1. Build the frontend: `cd frontend && bun run build`
2. Run the backend: `cd backend && cargo run --release`
3. The Rust server will serve the static files from `frontend/dist` at `http://localhost:3000`.

## 🛠 Tech Stack

* **Language:** Rust & TypeScript
* **Database:** SQLite (via SQLx)
* **UI Framework:** React + Tailwind CSS v4 + shadcn/ui
* **Web Server:** Axum

