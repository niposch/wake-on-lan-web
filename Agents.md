# 🕵️ Device Agents

To allow the web interface to **turn off** computers (which Wake-on-LAN cannot do), we use a lightweight "Agent" installed on the target machine.

## Design Specification

The agent is a small Rust binary that runs as a system service (Windows Service / systemd unit).

### Responsibilities
1.  **Listen:** Binds to a TCP port (default `3001`).
2.  **Authenticate:** Verifies the request comes from the WoL Backend (via shared secret or simple header token).
3.  **Execute:** Runs the OS-level shutdown command.

### Protocol
* **Transport:** HTTP (REST)
* **Endpoint:** `POST /shutdown`
* **Headers:** `Authorization: Bearer <SHARED_SECRET>`

### Implementation Plan (Rust)

We will use `axum` (minimal features) or raw `TcpListener` to keep the binary size tiny (<5MB).

**Shutdown Commands:**
* **Windows:** `shutdown /s /t 0`
* **Linux:** `shutdown -h now` (requires sudo/root permissions)
* **macOS:** `sudo shutdown -h now`

### Security Considerations
* The agent should bind to `0.0.0.0` to accept LAN connections.
* **Firewall:** You must allow traffic on port `3001` on the target machine.
* **Authentication:** Crucial. Without a token, anyone on the LAN could shut down your PC by hitting the endpoint.

### Future Ideas
* **Heartbeat:** The agent could ping the server every minute to report "I am Alive," replacing the need for ICMP pings.
* **Restart:** Add a `/reboot` endpoint.
