# Wake-on-LAN Web Manager

A self-hosted web application to manage and wake devices on your local network.

## Features

- **Dashboard:** View device status (Online/Offline) and wake/shutdown them.
- **Device Management:** Add, edit, and delete devices (MAC address, IP, etc.).
- **User Management:** Admin role can create users, reset passwords, and manage permissions.
- **Authentication:** JWT-based login with forced password change on first login.
- **Agent Integration:** Optional agent for remote shutdown (Windows/Linux/macOS).
- **Pinger:** Background task checks device availability every minute.

## Getting Started

### Prerequisites

- Rust (latest stable)
- Node.js & npm
- SQLite3

### Backend Setup

1.  Navigate to `backend`:
    ```bash
    cd backend
    ```
2.  Install dependencies and build:
    ```bash
    cargo build --release
    ```
3.  Run migrations (creates `wol.db`):
    ```bash
    # Install sqlx-cli if needed: cargo install sqlx-cli
    sqlx migrate run
    ```
4.  Start the server and initialize the admin user:
    ```bash
    # First run to set admin password
    cargo run -- --admin-password "secret123"
    ```
    *Subsequent runs can omit the `--admin-password` flag unless you want to reset it.*

The API will be available at `http://localhost:3000`.
Swagger UI: `http://localhost:3000/swagger/`

### Frontend Setup
1.  Navigate to `frontend/`.
2.  Install dependencies:
    ```bash
    npm install
    ```
3.  Start the development server:
    ```bash
    npm run dev
    ```

## Demo Mode

YouCan run the frontend in a standalone "Demo Mode" without the Rust backend. This mode uses a mock API service that simulates all backend functionality (authentication, database operations, device state) within the browser using LocalStorage.

To start Demo Mode:

1.  Navigate to `frontend/`.
2.  Run:
    ```bash
    npm run demo
    ```

**Default Credentials:**
-   **Admin:** `admin` / `admin`
-   **User:** `user` / `user`

*Note: Data changes in demo mode are persisted in your browser's LocalStorage. Clear your browser data to reset the demo state.*

## API Documentation
When the backend is running, the Swagger UI is available at:
`http://localhost:3000/swagger`

### Running in Production

To serve the frontend from the backend:

1.  Create a `static_files` directory in the `backend` root.
2.  Copy the contents of `frontend/dist` to `backend/static_files`.
3.  Run the backend binary.

## Development

- **Backend:** `cargo run` (Port 3000)
- **Frontend:** `npm run dev` (Port 5173 - Proxy configured to forward API calls to 3000)

## Security Note

- The background pinger requires raw socket privileges on some OSs (like Linux). You may need to run the binary with `sudo` or set capabilities: `setcap cap_net_raw+ep ./target/release/backend`.
- Windows usually allows ping without special privileges if run as a standard user, or requires Admin if using raw sockets depending on the implementation of `surge-ping`.

## Architecture

- **Backend:** Axum (Rust), SQLx (SQLite), Tokio, JsonWebToken.
- **Frontend:** React, Vite, Shadcn UI, Tailwind CSS, Axios.
