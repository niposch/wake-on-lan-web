use crate::db::AppState;
use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
};
use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use chrono::NaiveDateTime;
use rand_core::OsRng;
use rand::distr::{Alphanumeric, SampleString};
use serde::{Deserialize, Serialize};
use utoipa::{OpenApi, ToSchema};

// ==========================================
// 1. DTOs (Data Models for JSON)
// ==========================================

#[derive(Deserialize, ToSchema)]
pub struct CreateUserRequest {
    pub username: String,
}

#[derive(Deserialize, ToSchema)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize, ToSchema)]
pub struct UpdateRoleRequest {
    pub role: String, // 'admin' or 'user'
}

#[derive(Deserialize, ToSchema)]
pub struct ResetPasswordRequest {
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
    // In the future, you will add "token": String here for JWT
}

// ==========================================
// 2. HELPER FUNCTIONS (Service Logic)
// ==========================================

fn hash_password(password: &str) -> Result<String, String> {
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
    State(state): State<AppState>,
    Json(payload): Json<CreateUserRequest>,
) -> impl IntoResponse {
    // generate a random password with 8 alphanumeric characters
    let password = Alphanumeric.sample_string(&mut rand::rng(), 8);

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
            RETURNING id as "id!", username, role, last_login_at, force_password_change
        "#,
        payload.username,
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
    // 1. Fetch user by username
    let user = sqlx::query!(
        r#"SELECT id as "id!", username, password_hash, role, last_login_at, force_password_change 
         FROM users WHERE username = ?"#,
        payload.username
    )
    .fetch_optional(&state.db)
    .await
    .unwrap_or(None);

    let user = match user {
        Some(u) => u,
        None => return (StatusCode::UNAUTHORIZED, "Invalid credentials").into_response(),
    };

    // 2. Check if password change is required (before password verification)
    if user.force_password_change {
        // Don't verify password or increment failed attempts - change is required
        return (StatusCode::FORBIDDEN, "Password change required").into_response();
    }

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

    // 5. Return User Info
    let response = LoginResponse {
        message: "Login successful".to_string(),
        user: UserResponse {
            id: user.id,
            username: user.username,
            role: user.role,
            last_login_at: user.last_login_at,
            force_password_change: user.force_password_change,
        },
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
pub async fn list_users(State(state): State<AppState>) -> impl IntoResponse {
    let users = sqlx::query_as!(
        UserResponse,
        "SELECT id, username, role, last_login_at, force_password_change FROM users"
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
    State(state): State<AppState>,
    Path(user_id): Path<i64>,
    Json(payload): Json<UpdateRoleRequest>,
) -> impl IntoResponse {
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

/// POST /api/users/:id/reset-password
#[utoipa::path(
    post,
    path = "/api/users/{id}/reset-password",
    params(
        ("id" = i64, Path, description = "User ID")
    ),
    request_body = ResetPasswordRequest,
    tag = "users",
    responses(
        (status = 200, description = "Password reset"),
        (status = 401, description = "Invalid credentials"),
        (status = 404, description = "User not found")
    )
)]
pub async fn admin_reset_password(
    State(state): State<AppState>,
    Path(user_id): Path<i64>,
    Json(payload): Json<ResetPasswordRequest>,
) -> impl IntoResponse {

    let user = sqlx::query!("SELECT password_hash FROM users WHERE id = ?", user_id)
        .fetch_optional(&state.db)
        .await
        .unwrap_or(None);

    let user = match user {
        Some(u) => u,
        None => return (StatusCode::NOT_FOUND, "User not found").into_response(),
    };

    if !verify_password(&payload.old_password, &user.password_hash) {
        return (StatusCode::UNAUTHORIZED, "Invalid credentials").into_response();
    }

    let password_hash = match hash_password(&payload.new_password) {
        Ok(h) => h,
        Err(_) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to hash password").into_response();
        }
    };

    let result = sqlx::query!(
        "UPDATE users SET password_hash = ?, failed_login_attempts = 0, last_login_at = CURRENT_TIMESTAMP, force_password_change = 0 WHERE id = ?",
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
            Json(serde_json::json!({
                "message": "Password reset successfully",
            })),
        )
            .into_response(),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to reset password",
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
    State(state): State<AppState>,
    Path(user_id): Path<i64>,
) -> impl IntoResponse {
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

// 1. Bundle everything in this module
#[derive(OpenApi)]
#[openapi(
    paths(
        create_user,
        login,
        list_users,
        update_role,
        admin_reset_password,
        delete_user
    ),
    components(
        schemas(
            CreateUserRequest,
            LoginRequest,
            LoginResponse,
            UserResponse,
            UpdateRoleRequest,
            ResetPasswordRequest
        )
    ),
    tags(
        (name = "users", description = "User management endpoints")
    )
)]
pub struct UserApi;
