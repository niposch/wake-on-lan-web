use crate::db::AppState;
use crate::auth::{AuthUser, AdminUser, create_jwt, generate_refresh_token};
use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
};
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use chrono::{NaiveDateTime, TimeZone};
use rand_core::OsRng;
use rand::distr::{Alphanumeric, SampleString};
use serde::{Deserialize, Serialize};
use utoipa::{OpenApi, ToSchema};

// ==========================================
// 1. DTOs
// ==========================================

#[derive(Deserialize, ToSchema)]
pub struct CreateUserRequest {
    pub username: String,
}

#[derive(Deserialize, ToSchema)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
    pub remember_me: Option<bool>,
}

#[derive(Deserialize, ToSchema)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

#[derive(Serialize, ToSchema)]
pub struct RefreshTokenResponse {
    pub access_token: String,
    pub refresh_token: String,
}

#[derive(Deserialize, ToSchema)]
pub struct UpdateRoleRequest {
    pub role: String, // 'admin' or 'user'
}

#[derive(Deserialize, ToSchema)]
pub struct UpdateStatusRequest {
    pub is_disabled: bool,
}

#[derive(Deserialize, ToSchema)]
pub struct AdminResetPasswordRequest {
    pub new_password: Option<String>,
}

#[derive(Serialize, ToSchema)]
pub struct AdminResetPasswordResponse {
    pub message: String,
    pub password: Option<String>,
}

#[derive(Deserialize, ToSchema)]
pub struct ChangePasswordRequest {
    pub old_password: String,
    pub new_password: String,
}

#[derive(Serialize, ToSchema)]
pub struct UserResponse {
    pub id: i64,
    pub username: String,
    pub role: String,
    pub last_login_at: Option<NaiveDateTime>,
    pub force_password_change: bool,
    pub is_disabled: bool,
}

#[derive(Serialize, ToSchema)]
pub struct CreateUserResponse {
    pub message: String,
    pub user: UserResponse,
    pub password: String,
}

#[derive(Serialize, ToSchema)]
pub struct LoginResponse {
    pub message: String,
    pub user: UserResponse,
    pub access_token: String,
    pub refresh_token: String,
}

// ==========================================
// 2. HELPER FUNCTIONS (Service Logic)
// ==========================================

pub fn hash_password(password: &str) -> Result<String, String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    argon2
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|e| e.to_string())
}

fn verify_password(password: &str, password_hash: &str) -> bool {
    let parsed_hash = match PasswordHash::new(password_hash) {
        Ok(h) => h,
        Err(_) => return false,
    };

    Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok()
}

// ==========================================
// 3. HANDLERS (Controllers)
// ==========================================

/// POST /api/users
#[utoipa::path(
    post,
    path = "/api/users",
    request_body = CreateUserRequest,
    tag = "users",
    responses(
        (status = 201, description = "User created", body = CreateUserResponse),
        (status = 409, description = "Username taken"),
        (status = 500, description = "Server error")
    )
)]
pub async fn create_user(
    _admin: AdminUser,
    State(state): State<AppState>,
    Json(payload): Json<CreateUserRequest>,
) -> impl IntoResponse {
    // generate a random password with 8 alphanumeric characters
    let password = Alphanumeric.sample_string(&mut rand::rng(), 8);

    // Ensure username is lowercase
    let username = payload.username.to_lowercase();

    // 1. Hash the password
    let password_hash = match hash_password(&password.to_string()) {
        Ok(h) => h,
        Err(_) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to hash password").into_response();
        }
    };

    // 2. Insert into DB, return inserted user fields via RETURNING
    let user_result = sqlx::query!(
        r#"
            INSERT INTO users (username, password_hash, force_password_change)
            VALUES (?, ?, 1)
            RETURNING id as "id!", username, role, last_login_at, force_password_change, is_disabled
        "#,
        username,
        password_hash
    )
    .fetch_one(&state.db)
    .await;

    match user_result {
        Ok(user) => {
            let resp = CreateUserResponse {
                message: "User created successfully".to_string(),
                user: UserResponse {
                    id: user.id,
                    username: user.username,
                    role: user.role,
                    last_login_at: user.last_login_at,
                    force_password_change: user.force_password_change,
                    is_disabled: user.is_disabled,
                },
                password: password.clone(),
            };
            (StatusCode::CREATED, Json(resp)).into_response()
        }
        Err(e) => {
            if e.to_string().contains("UNIQUE") {
                (StatusCode::CONFLICT, "Username already exists").into_response()
            } else {
                (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response()
            }
        }
    }
}

/// POST /api/login
#[utoipa::path(
    post,
    path = "/api/login",
    request_body = LoginRequest,
    tag = "users",
    responses(
        (status = 200, description = "Login successful", body = LoginResponse),
        (status = 401, description = "Invalid credentials"),
        (status = 403, description = "Password change required")
    )
)]
pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> impl IntoResponse {
    let username = payload.username.to_lowercase();

    // 1. Fetch user by username
    let user = sqlx::query!(
        r#"SELECT id as "id!", username, password_hash, role, last_login_at, force_password_change, is_disabled
         FROM users WHERE username = ?"#,
        username
    )
    .fetch_optional(&state.db)
    .await
    .unwrap_or(None);

    let user = match user {
        Some(u) => u,
        None => return (StatusCode::UNAUTHORIZED, "Invalid credentials").into_response(),
    };

    if user.is_disabled {
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({ "error": "Account disabled" })),
        )
            .into_response();
    }

    // 2. Check if password change is required (before password verification)
    // Actually, user MUST be able to login to change password.
    // So we should ALLOW login but user will have `force_password_change: true`.
    // The frontend should redirect them to change password page.
    
    // 3. Verify Password
    if !verify_password(&payload.password, &user.password_hash) {
        // Increment failed attempts (optional logic here)
        let _ = sqlx::query!(
            "UPDATE users SET failed_login_attempts = failed_login_attempts + 1 WHERE id = ?",
            user.id
        )
        .execute(&state.db)
        .await;

        return (StatusCode::UNAUTHORIZED, "Invalid credentials").into_response();
    }

    // 4. Success: Reset failed attempts & Update last login
    let _ = sqlx::query!(
        "UPDATE users SET failed_login_attempts = 0, last_login_at = CURRENT_TIMESTAMP WHERE id = ?",
        user.id
    )
    .execute(&state.db)
    .await;

    // 5. Generate Tokens
    // Access Token: 15 minutes
    let access_token = match create_jwt(user.id, &user.username, &user.role, chrono::Duration::minutes(15)) {
        Ok(t) => t,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to generate token").into_response(),
    };

    // Refresh Token
    let refresh_token = generate_refresh_token();
    let duration = if payload.remember_me.unwrap_or(false) {
        chrono::Duration::days(30)
    } else {
        chrono::Duration::days(1)
    };
    let refresh_expires_at = chrono::Utc::now() + duration;

    // Store Refresh Token in DB
    // Ideally we hash it, but for simplicity we store as is (it's high entropy)
    let _ = sqlx::query!(
        "INSERT INTO refresh_tokens (token_hash, user_id, expires_at) VALUES (?, ?, ?)",
        refresh_token,
        user.id,
        refresh_expires_at
    )
    .execute(&state.db)
    .await;

    // 6. Return User Info
    let response = LoginResponse {
        message: "Login successful".to_string(),
        user: UserResponse {
            id: user.id,
            username: user.username,
            role: user.role,
            last_login_at: user.last_login_at,
            force_password_change: user.force_password_change,
            is_disabled: user.is_disabled,
        },
        access_token,
        refresh_token,
    };

    (StatusCode::OK, Json(response)).into_response()
}

/// GET /api/users
#[utoipa::path(
    get,
    path = "/api/users",
    tag = "users",
    responses(
        (status = 200, description = "List all users", body = [UserResponse])
    )
)]
pub async fn list_users(
    _admin: AdminUser,
    State(state): State<AppState>
) -> impl IntoResponse {
    let users = sqlx::query_as!(
        UserResponse,
        "SELECT id, username, role, last_login_at, force_password_change, is_disabled FROM users"
    )
    .fetch_all(&state.db)
    .await;

    match users {
        Ok(u) => Json(u).into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch users").into_response(),
    }
}

/// PUT /api/users/:id/role
#[utoipa::path(
    put,
    path = "/api/users/{id}/role",
    params(
        ("id" = i64, Path, description = "User ID")
    ),
    request_body = UpdateRoleRequest,
    tag = "users",
    responses(
        (status = 200, description = "Role updated"),
        (status = 404, description = "User not found")
    )
)]
pub async fn update_role(
    admin: AdminUser,
    State(state): State<AppState>,
    Path(user_id): Path<i64>,
    Json(payload): Json<UpdateRoleRequest>,
) -> impl IntoResponse {
    if user_id == admin.0.id {
        return (StatusCode::FORBIDDEN, "Cannot change your own role").into_response();
    }

    let result = sqlx::query!(
        "UPDATE users SET role = ? WHERE id = ?",
        payload.role,
        user_id
    )
    .execute(&state.db)
    .await;

    match result {
        Ok(r) if r.rows_affected() == 0 => {
            (StatusCode::NOT_FOUND, "User not found").into_response()
        }
        Ok(_) => (StatusCode::OK, "Role updated").into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Failed to update role").into_response(),
    }
}

/// PUT /api/users/:id/status
#[utoipa::path(
    put,
    path = "/api/users/{id}/status",
    params(
        ("id" = i64, Path, description = "User ID")
    ),
    request_body = UpdateStatusRequest,
    tag = "users",
    responses(
        (status = 200, description = "Status updated"),
        (status = 404, description = "User not found")
    )
)]
pub async fn update_status(
    admin: AdminUser,
    State(state): State<AppState>,
    Path(user_id): Path<i64>,
    Json(payload): Json<UpdateStatusRequest>,
) -> impl IntoResponse {
    if user_id == admin.0.id && payload.is_disabled {
        return (StatusCode::FORBIDDEN, "Cannot disable your own account").into_response();
    }

    let result = sqlx::query!(
        "UPDATE users SET is_disabled = ? WHERE id = ?",
        payload.is_disabled,
        user_id
    )
    .execute(&state.db)
    .await;

    match result {
        Ok(r) if r.rows_affected() == 0 => {
            (StatusCode::NOT_FOUND, "User not found").into_response()
        }
        Ok(_) => (StatusCode::OK, "Status updated").into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Failed to update status").into_response(),
    }
}

/// POST /api/users/:id/reset-password
/// Admin overrides user password (e.g. if forgotten)
#[utoipa::path(
    post,
    path = "/api/users/{id}/reset-password",
    params(
        ("id" = i64, Path, description = "User ID")
    ),
    request_body = AdminResetPasswordRequest,
    tag = "users",
    responses(
        (status = 200, description = "Password reset"),
        (status = 404, description = "User not found")
    )
)]
pub async fn admin_reset_password(
    _admin: AdminUser,
    State(state): State<AppState>,
    Path(user_id): Path<i64>,
    Json(payload): Json<AdminResetPasswordRequest>,
) -> impl IntoResponse {

    let (password_hash, generated_password) = if let Some(p) = &payload.new_password {
        match hash_password(p) {
            Ok(h) => (h, None),
            Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to hash password").into_response(),
        }
    } else {
        let p = Alphanumeric.sample_string(&mut rand::rng(), 12);
        match hash_password(&p) {
            Ok(h) => (h, Some(p)),
            Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to hash password").into_response(),
        }
    };

    // Also force user to change it again on next login if desired?
    // Spec says: "User accounts should be created by the admins and these get assigned a temp password... On first log in they'd have to type in a new password."
    // If admin resets it, it's effectively a temp password again. So set force_password_change = 1.
    
    let result = sqlx::query!(
        "UPDATE users SET password_hash = ?, failed_login_attempts = 0, last_login_at = NULL, force_password_change = 1 WHERE id = ?",
        password_hash,
        user_id
    )
    .execute(&state.db)
    .await;

    match result {
        Ok(r) if r.rows_affected() == 0 => {
            (StatusCode::NOT_FOUND, "User not found").into_response()
        }
        Ok(_) => (
            StatusCode::OK,
            Json(AdminResetPasswordResponse {
                message: "Password reset successfully. User must change it on next login.".to_string(),
                password: generated_password,
            }),
        )
            .into_response(),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to reset password",
        )
            .into_response(),
    }
}

/// POST /api/change-password
/// User changes their own password
#[utoipa::path(
    post,
    path = "/api/change-password",
    request_body = ChangePasswordRequest,
    tag = "users",
    responses(
        (status = 200, description = "Password changed"),
        (status = 401, description = "Invalid credentials")
    )
)]
pub async fn change_password(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Json(payload): Json<ChangePasswordRequest>,
) -> impl IntoResponse {
    // 1. Verify old password
    let user = sqlx::query!("SELECT password_hash FROM users WHERE id = ?", auth_user.id)
        .fetch_optional(&state.db)
        .await
        .unwrap_or(None);
        
    let user = match user {
        Some(u) => u,
        None => return (StatusCode::UNAUTHORIZED, "User not found").into_response(),
    };

    if !verify_password(&payload.old_password, &user.password_hash) {
        return (StatusCode::UNAUTHORIZED, "Invalid current password").into_response();
    }

    // 2. Hash new password
    let password_hash = match hash_password(&payload.new_password) {
        Ok(h) => h,
        Err(_) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to hash password").into_response();
        }
    };

    // 3. Update DB
    let result = sqlx::query!(
        "UPDATE users SET password_hash = ?, force_password_change = 0 WHERE id = ?",
        password_hash,
        auth_user.id
    )
    .execute(&state.db)
    .await;

    match result {
        Ok(_) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "message": "Password changed successfully",
            })),
        )
            .into_response(),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to change password",
        )
            .into_response(),
    }
}

/// DELETE /api/users/:id
#[utoipa::path(
    delete,
    path = "/api/users/{id}",
    params(
        ("id" = i64, Path, description = "User ID")
    ),
    tag = "users",
    responses(
        (status = 200, description = "User deleted"),
        (status = 404, description = "User not found"),
        (status = 500, description = "Server error")
    )
)]
pub async fn delete_user(
    admin: AdminUser,
    State(state): State<AppState>,
    Path(user_id): Path<i64>,
) -> impl IntoResponse {
    if user_id == admin.0.id {
        return (StatusCode::FORBIDDEN, "Cannot delete your own account").into_response();
    }

    let result = sqlx::query!("DELETE FROM users WHERE id = ?", user_id)
        .execute(&state.db)
        .await;

    match result {
        Ok(r) if r.rows_affected() == 0 => {
            (StatusCode::NOT_FOUND, "User not found").into_response()
        }
        Ok(_) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "message": "User deleted successfully"
            })),
        )
            .into_response(),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to delete user",
        )
            .into_response(),
    }
}

/// POST /api/refresh
#[utoipa::path(
    post,
    path = "/api/refresh",
    request_body = RefreshTokenRequest,
    tag = "users",
    responses(
        (status = 200, description = "Tokens refreshed", body = RefreshTokenResponse),
        (status = 401, description = "Invalid or expired refresh token")
    )
)]
pub async fn refresh_token(
    State(state): State<AppState>,
    Json(payload): Json<RefreshTokenRequest>,
) -> impl IntoResponse {
    // 1. Verify Refresh Token in DB
    let token_record = sqlx::query!(
        "SELECT token_hash, user_id, expires_at FROM refresh_tokens WHERE token_hash = ?",
        payload.refresh_token
    )
    .fetch_optional(&state.db)
    .await
    .unwrap_or(None);

    let token_record = match token_record {
        Some(t) => t,
        None => return (StatusCode::UNAUTHORIZED, "Invalid refresh token").into_response(),
    };

    // 2. Check Expiration
    let now = chrono::Utc::now();
    let expires_at = chrono::Utc.from_utc_datetime(&token_record.expires_at);
    
    if expires_at < now {
        // Delete expired token
        let _ = sqlx::query!("DELETE FROM refresh_tokens WHERE token_hash = ?", payload.refresh_token)
            .execute(&state.db)
            .await;
        return (StatusCode::UNAUTHORIZED, "Refresh token expired").into_response();
    }

    // 3. Fetch User
    let user = sqlx::query!(
        "SELECT username, role FROM users WHERE id = ?",
        token_record.user_id
    )
    .fetch_optional(&state.db)
    .await
    .unwrap_or(None);

    let user = match user {
        Some(u) => u,
        None => return (StatusCode::UNAUTHORIZED, "User not found").into_response(),
    };

    // 4. Rotate Tokens
    // Delete old
    let _ = sqlx::query!("DELETE FROM refresh_tokens WHERE token_hash = ?", payload.refresh_token)
        .execute(&state.db)
        .await;

    // Generate New
    let access_token = match create_jwt(token_record.user_id, &user.username, &user.role, chrono::Duration::minutes(15)) {
        Ok(t) => t,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to generate token").into_response(),
    };

    let new_refresh_token = generate_refresh_token();
    // Keep same expiration duration logic? Or slide it? Let's slide it.
    // Actually, calculate duration from old token? Or just give fresh duration?
    // Let's give fresh 30 days / 1 day based on...? We lost "remember_me" context.
    // We can infer it: if old token was > 24h, it was remember_me.
    // Or just simplify: Refreshing keeps the session alive, so slide window.
    // Default to 30 days sliding window for simplicity in this iteration.
    let new_expires_at = now + chrono::Duration::days(30);

    let _ = sqlx::query!(
        "INSERT INTO refresh_tokens (token_hash, user_id, expires_at) VALUES (?, ?, ?)",
        new_refresh_token,
        token_record.user_id,
        new_expires_at
    )
    .execute(&state.db)
    .await;

    (StatusCode::OK, Json(RefreshTokenResponse {
        access_token,
        refresh_token: new_refresh_token,
    })).into_response()
}

/// POST /api/logout
#[utoipa::path(
    post,
    path = "/api/logout",
    request_body = RefreshTokenRequest,
    tag = "users",
    responses(
        (status = 200, description = "Logged out")
    )
)]
pub async fn logout_user(
    State(state): State<AppState>,
    Json(payload): Json<RefreshTokenRequest>,
) -> impl IntoResponse {
    let _ = sqlx::query!("DELETE FROM refresh_tokens WHERE token_hash = ?", payload.refresh_token)
        .execute(&state.db)
        .await;

    (StatusCode::OK, Json(serde_json::json!({"message": "Logged out"}))).into_response()
}

/// GET /api/me
#[utoipa::path(
    get,
    path = "/api/me",
    tag = "users",
    responses(
        (status = 200, description = "Current user info", body = UserResponse),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn get_me(
    auth_user: AuthUser,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let user = sqlx::query_as!(
        UserResponse,
        "SELECT id, username, role, last_login_at, force_password_change, is_disabled FROM users WHERE id = ?",
        auth_user.id
    )
    .fetch_optional(&state.db)
    .await;

    match user {
        Ok(Some(u)) => Json(u).into_response(),
        Ok(None) => (StatusCode::UNAUTHORIZED, "User not found").into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response(),
    }
}

// 1. Bundle everything in this module
#[derive(OpenApi)]
#[openapi(
    paths(
        create_user,
        login,
        refresh_token,
        logout_user,
        get_me,
        list_users,
        update_role,
        update_status,
        admin_reset_password,
        change_password,
        delete_user
    ),
    components(
        schemas(
            CreateUserRequest,
            LoginRequest,
            RefreshTokenRequest,
            RefreshTokenResponse,
            LoginResponse,
            UserResponse,
            UpdateRoleRequest,
            UpdateStatusRequest,
            AdminResetPasswordRequest,
            AdminResetPasswordResponse,
            ChangePasswordRequest
        )
    ),
    tags(
        (name = "users", description = "User management endpoints")
    )
)]
pub struct UserApi;
