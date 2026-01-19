use crate::db::AppState;
use crate::auth::{AuthUser, AdminUser};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use utoipa::{OpenApi, ToSchema};
use wake_on_lan::MagicPacket;

// ==========================================
// 1. DTOs
// ==========================================

#[derive(Deserialize, ToSchema)]
pub struct CreateDeviceRequest {
    pub name: String,
    pub mac_address: String,
    pub ip_address: Option<String>,
    pub broadcast_addr: Option<String>,
    pub icon: Option<String>,
}

#[derive(Deserialize, ToSchema)]
pub struct UpdateDeviceRequest {
    pub name: Option<String>,
    pub mac_address: Option<String>,
    pub ip_address: Option<String>,
    pub broadcast_addr: Option<String>,
    pub icon: Option<String>,
}

#[derive(Serialize, ToSchema)]
pub struct DeviceResponse {
    pub id: i64,
    pub name: String,
    pub mac_address: String,
    pub ip_address: Option<String>,
    pub broadcast_addr: Option<String>,
    pub icon: Option<String>,
    pub is_online: bool,
    pub last_seen_at: Option<chrono::NaiveDateTime>,
}

// ==========================================
// 2. HANDLERS
// ==========================================

/// GET /api/devices
#[utoipa::path(
    get,
    path = "/api/devices",
    tag = "devices",
    responses(
        (status = 200, description = "List all devices", body = [DeviceResponse])
    )
)]
pub async fn list_devices(
    _auth: AuthUser,
    State(state): State<AppState>
) -> impl IntoResponse {
    let devices = sqlx::query!(
        r#"SELECT 
            id, name, mac_address, ip_address, broadcast_addr, 
            icon, is_online, last_seen_at 
           FROM devices"#
    )
    .fetch_all(&state.db)
    .await;

    match devices {
        Ok(rows) => {
            let res: Vec<DeviceResponse> = rows.into_iter().map(|row| DeviceResponse {
                id: row.id,
                name: row.name,
                mac_address: row.mac_address,
                ip_address: row.ip_address,
                broadcast_addr: row.broadcast_addr,
                icon: row.icon,
                is_online: row.is_online.unwrap_or(false),
                last_seen_at: row.last_seen_at,
            }).collect();
            Json(res).into_response()
        },
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch devices").into_response(),
    }
}

/// POST /api/devices
#[utoipa::path(
    post,
    path = "/api/devices",
    request_body = CreateDeviceRequest,
    tag = "devices",
    responses(
        (status = 201, description = "Device created", body = DeviceResponse),
        (status = 500, description = "Server error")
    )
)]
pub async fn create_device(
    _admin: AdminUser,
    State(state): State<AppState>,
    Json(payload): Json<CreateDeviceRequest>,
) -> impl IntoResponse {
    let broadcast_addr = payload.broadcast_addr.unwrap_or_else(|| "255.255.255.255".to_string());
    
    let result = sqlx::query!(
        r#"
            INSERT INTO devices (name, mac_address, ip_address, broadcast_addr, icon)
            VALUES (?, ?, ?, ?, ?)
            RETURNING id as "id!", name, mac_address, ip_address, broadcast_addr, icon, is_online, last_seen_at
        "#,
        payload.name,
        payload.mac_address,
        payload.ip_address,
        broadcast_addr,
        payload.icon
    )
    .fetch_one(&state.db)
    .await;

    match result {
        Ok(dev) => {
            let resp = DeviceResponse {
                id: dev.id,
                name: dev.name,
                mac_address: dev.mac_address,
                ip_address: dev.ip_address,
                broadcast_addr: dev.broadcast_addr,
                icon: dev.icon,
                is_online: dev.is_online,
                last_seen_at: dev.last_seen_at,
            };
            (StatusCode::CREATED, Json(resp)).into_response()
        }
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Failed to create device").into_response(),
    }
}

/// PUT /api/devices/:id
#[utoipa::path(
    put,
    path = "/api/devices/{id}",
    params(
        ("id" = i64, Path, description = "Device ID")
    ),
    request_body = UpdateDeviceRequest,
    tag = "devices",
    responses(
        (status = 200, description = "Device updated", body = DeviceResponse),
        (status = 404, description = "Device not found"),
        (status = 500, description = "Server error")
    )
)]
pub async fn update_device(
    _admin: AdminUser,
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateDeviceRequest>,
) -> impl IntoResponse {
    let result = sqlx::query!(
        r#"
            UPDATE devices 
            SET 
                name = COALESCE(?, name),
                mac_address = COALESCE(?, mac_address),
                ip_address = COALESCE(?, ip_address),
                broadcast_addr = COALESCE(?, broadcast_addr),
                icon = COALESCE(?, icon)
            WHERE id = ?
            RETURNING id as "id!", name, mac_address, ip_address, broadcast_addr, icon, is_online, last_seen_at
        "#,
        payload.name,
        payload.mac_address,
        payload.ip_address,
        payload.broadcast_addr,
        payload.icon,
        id
    )
    .fetch_optional(&state.db)
    .await;

    match result {
        Ok(Some(dev)) => {
            let resp = DeviceResponse {
                id: dev.id,
                name: dev.name,
                mac_address: dev.mac_address,
                ip_address: dev.ip_address,
                broadcast_addr: dev.broadcast_addr,
                icon: dev.icon,
                is_online: dev.is_online.unwrap_or(false),
                last_seen_at: dev.last_seen_at,
            };
            (StatusCode::OK, Json(resp)).into_response()
        },
        Ok(None) => (StatusCode::NOT_FOUND, "Device not found").into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Failed to update device").into_response(),
    }
}

/// DELETE /api/devices/:id
#[utoipa::path(
    delete,
    path = "/api/devices/{id}",
    params(
        ("id" = i64, Path, description = "Device ID")
    ),
    tag = "devices",
    responses(
        (status = 200, description = "Device deleted"),
        (status = 404, description = "Device not found")
    )
)]
pub async fn delete_device(
    _admin: AdminUser,
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    let result = sqlx::query!("DELETE FROM devices WHERE id = ?", id)
        .execute(&state.db)
        .await;

    match result {
        Ok(r) if r.rows_affected() == 0 => (StatusCode::NOT_FOUND, "Device not found").into_response(),
        Ok(_) => (StatusCode::OK, "Device deleted").into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Failed to delete device").into_response(),
    }
}

/// POST /api/devices/:id/wake
#[utoipa::path(
    post,
    path = "/api/devices/{id}/wake",
    params(
        ("id" = i64, Path, description = "Device ID")
    ),
    tag = "devices",
    responses(
        (status = 200, description = "Wake signal sent"),
        (status = 404, description = "Device not found"),
        (status = 500, description = "Failed to send packet")
    )
)]
pub async fn wake_device(
    _auth: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    // 1. Get device details
    let device = sqlx::query!(
        "SELECT mac_address, broadcast_addr FROM devices WHERE id = ?",
        id
    )
    .fetch_optional(&state.db)
    .await;

    let device = match device {
        Ok(Some(d)) => d,
        Ok(None) => return (StatusCode::NOT_FOUND, "Device not found").into_response(),
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response(),
    };

    // 2. Parse MAC address
    let mac_bytes: Vec<u8> = device.mac_address
        .split(|c| c == ':' || c == '-')
        .filter_map(|s| u8::from_str_radix(s, 16).ok())
        .collect();

    if mac_bytes.len() != 6 {
         return (StatusCode::BAD_REQUEST, "Invalid MAC address format in DB").into_response();
    }

    let mut mac_array = [0u8; 6];
    mac_array.copy_from_slice(&mac_bytes);

    let magic_packet = MagicPacket::new(&mac_array);
    
    // 3. Send Packet
    let res = if let Some(b_addr) = device.broadcast_addr {
         // Try to send to specific broadcast address + port 9
         magic_packet.send_to((b_addr.as_str(), 9), ("0.0.0.0", 0))
    } else {
         magic_packet.send()
    };

    match res {
        Ok(_) => (StatusCode::OK, "Wake signal sent").into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to send WoL: {}", e)).into_response(),
    }
}

/// POST /api/devices/:id/shutdown
#[utoipa::path(
    post,
    path = "/api/devices/{id}/shutdown",
    params(
        ("id" = i64, Path, description = "Device ID")
    ),
    tag = "devices",
    responses(
        (status = 200, description = "Shutdown signal sent"),
        (status = 404, description = "Device not found"),
        (status = 502, description = "Failed to contact agent")
    )
)]
pub async fn shutdown_device(
    _auth: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    // 1. Get device details
    let device = sqlx::query!(
        "SELECT ip_address FROM devices WHERE id = ?",
        id
    )
    .fetch_optional(&state.db)
    .await;

    let device = match device {
        Ok(Some(d)) => d,
        Ok(None) => return (StatusCode::NOT_FOUND, "Device not found").into_response(),
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response(),
    };

    let ip = match device.ip_address {
        Some(ip) => ip,
        None => return (StatusCode::BAD_REQUEST, "Device has no IP address").into_response(),
    };

    // 2. Call the agent
    let client = reqwest::Client::new();
    // Assuming the agent runs on port 3001 and has a /shutdown endpoint
    // We should probably store the agent port in the DB or config, but hardcoding 3001 for now as per spec
    let url = format!("http://{}:3001/shutdown", ip);
    
    // NOTE: Auth token/secret is not yet implemented in DB.
    // For now we'll send a dummy token or no token if the agent doesn't enforce it yet.
    // Spec says: Authorization: Bearer <SHARED_SECRET>
    // Let's assume a default secret for now or skip if not ready.
    
    let res = client.post(&url)
        // .header("Authorization", "Bearer secret") 
        .send()
        .await;

    match res {
        Ok(r) => {
            if r.status().is_success() {
                 (StatusCode::OK, "Shutdown signal sent").into_response()
            } else {
                 (StatusCode::BAD_GATEWAY, "Agent returned error").into_response()
            }
        }
        Err(_) => (StatusCode::BAD_GATEWAY, "Failed to contact agent").into_response(),
    }
}

// 1. Bundle everything in this module
#[derive(OpenApi)]
#[openapi(
    paths(
        list_devices,
        create_device,
        update_device,
        delete_device,
        wake_device,
        shutdown_device
    ),
    components(
        schemas(
            CreateDeviceRequest,
            UpdateDeviceRequest,
            DeviceResponse
        )
    ),
    tags(
        (name = "devices", description = "Device management endpoints")
    )
)]
pub struct DeviceApi;
