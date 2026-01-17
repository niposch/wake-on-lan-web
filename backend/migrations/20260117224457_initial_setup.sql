-- 1. USERS TABLE
-- Stores login info, roles, and security metadata
CREATE TABLE users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    username TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,       -- Argon2 hash (includes salt within the string)
    role TEXT NOT NULL DEFAULT 'user', -- 'admin' or 'user'
    
    -- Security Metadata
    force_password_change BOOLEAN NOT NULL DEFAULT 0, -- Set to 1 when admin resets password
    failed_login_attempts INTEGER NOT NULL DEFAULT 0, -- Reset to 0 on successful login
    last_login_at DATETIME,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- 2. DEVICES TABLE
-- The core WoL configuration
CREATE TABLE devices (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    mac_address TEXT NOT NULL,         -- Format: AA:BB:CC:DD:EE:FF
    ip_address TEXT,                   -- Optional, for ping checks
    broadcast_addr TEXT DEFAULT '255.255.255.255', -- Default broadcast, changeable for subnets
    
    -- UI & State
    icon TEXT,                         -- String identifier for frontend icons (e.g., 'desktop', 'server')
    is_online BOOLEAN DEFAULT 0,       -- Caching current state for fast UI rendering
    last_seen_at DATETIME,             -- Timestamp of last successful ping
    
    created_by INTEGER,                -- Admin who created it (Nullable in case user is deleted)
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (created_by) REFERENCES users(id) ON DELETE SET NULL
);

-- 3. DEVICE EVENTS (Activity Log)
-- Tracks history: "User X turned on Device Y" or "Device Y went offline"
CREATE TABLE device_events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    device_id INTEGER NOT NULL,
    user_id INTEGER,                   -- NULL if system event (e.g. background pinger detected offline)
    
    event_type TEXT NOT NULL,          -- 'wake', 'shutdown', 'ping_online', 'ping_offline'
    description TEXT,                  -- Optional details (e.g. "Triggered by Admin via API")
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    FOREIGN KEY (device_id) REFERENCES devices(id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE SET NULL
);

-- Indexes for performance
CREATE INDEX idx_users_username ON users(username);
CREATE INDEX idx_devices_mac ON devices(mac_address);
CREATE INDEX idx_events_device ON device_events(device_id);