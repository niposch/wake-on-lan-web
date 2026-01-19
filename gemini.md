# Wake-on-LAN Web

A web-based application to manage and wake devices on a local network, featuring a Rust backend and a React frontend.

## Project Structure

The project is organized as a monorepo with two main directories:

-   `backend/`: Rust application handling API, database, and network logic.
-   `frontend/`: React application providing the user interface.

## Tech Stack

### Backend
-   **Language:** Rust
-   **Framework:** [Axum](https://github.com/tokio-rs/axum)
-   **Database:** SQLite (via [SQLx](https://github.com/launchbadge/sqlx))
-   **Async Runtime:** Tokio
-   **Documentation:** OpenAPI/Swagger (via [Utoipa](https://github.com/juhaku/utoipa))
-   **Key Libraries:** `wake-on-lan` (Magic Packet), `surge-ping` (ICMP Ping), `argon2` (Password Hashing), `jsonwebtoken` (Auth).

### Frontend
-   **Framework:** React 19 (via [Vite](https://vitejs.dev/))
-   **Language:** TypeScript
-   **Styling:** Tailwind CSS 4
-   **UI Components:** Radix UI Primitives, Lucide Icons, Sonner (Toasts)
-   **Routing:** React Router DOM 7
-   **State/Data:** Axios, React Context (Auth).

## Features

### Authentication & Authorization
-   **JWT-based Authentication**: Secure login flow.
-   **Role-Based Access Control (RBAC)**:
    -   **Admin**: Full access to user management and device operations.
    -   **User**: Can view and operate devices (wake/shutdown).
-   **Security**: Force password change mechanism for new users or after admin resets.

### Device Management
-   **Wake-on-LAN**: Send magic packets to wake up devices using their MAC address.
-   **Status Monitoring**: Background service pings devices every 60 seconds to update online/offline status.
-   **Device Inventory**: Add, remove, and list devices with details like Name, MAC Address, IP, and Broadcast Address.
-   **Remote Shutdown**: Support for triggering shutdown commands (endpoint available).

### User Management
-   Admin-only interface to:
    -   Create new users.
    -   Delete users.
    -   Update user roles.
    -   Reset user passwords.

## Getting Started

### Backend Setup
1.  Navigate to `backend/`.
2.  Install dependencies and setup DB:
    ```bash
    cargo sqlx database setup
    ```
3.  Run the server:
    ```bash
    cargo run
    ```
    *Optionally initialize an admin user:*
    ```bash
    cargo run -- --admin-password <your-password>
    ```

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

## API Documentation
When the backend is running, the Swagger UI is available at:
`http://localhost:3000/swagger`
