use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Response},
    Json, RequestPartsExt,
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::env;
use std::sync::OnceLock;
use crate::db::AppState;

static JWT_SECRET: OnceLock<String> = OnceLock::new();

pub fn get_jwt_secret() -> &'static str {
    JWT_SECRET.get_or_init(|| {
        env::var("JWT_SECRET").unwrap_or_else(|_| {
            println!("WARNING: JWT_SECRET not set, using random secret. Tokens will be invalid after restart.");
            use rand::distr::{Alphanumeric, SampleString};
            Alphanumeric.sample_string(&mut rand::rng(), 32)
        })
    })
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String, // username
    pub uid: i64,    // user id
    pub role: String, // 'admin' or 'user'
    pub exp: usize,
}

pub fn create_jwt(uid: i64, username: &str, role: &str, duration: chrono::Duration) -> Result<String, jsonwebtoken::errors::Error> {
    let expiration = chrono::Utc::now()
        .checked_add_signed(duration)
        .expect("valid timestamp")
        .timestamp();

    let claims = Claims {
        sub: username.to_owned(),
        uid,
        role: role.to_owned(),
        exp: expiration as usize,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(get_jwt_secret().as_bytes()),
    )
}

pub fn generate_refresh_token() -> String {
    use rand::distr::{Alphanumeric, SampleString};
    Alphanumeric.sample_string(&mut rand::rng(), 64)
}

pub struct AuthUser {
    pub id: i64,
    pub username: String,
    pub role: String,
}

// #[async_trait]
impl FromRequestParts<AppState> for AuthUser {
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        // Extract the token from the authorization header
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| AuthError::MissingCredentials)?;

        // Decode the user data
        let token_data = decode::<Claims>(
            bearer.token(),
            &DecodingKey::from_secret(get_jwt_secret().as_bytes()),
            &Validation::default(),
        )
        .map_err(|_| AuthError::InvalidToken)?;

        // Check if user is disabled
        let user = sqlx::query!("SELECT is_disabled FROM users WHERE id = ?", token_data.claims.uid)
            .fetch_optional(&state.db)
            .await
            .map_err(|_| AuthError::DatabaseError)?;

        match user {
            Some(u) if u.is_disabled => Err(AuthError::AccountDisabled),
            Some(_) => Ok(AuthUser {
                id: token_data.claims.uid,
                username: token_data.claims.sub,
                role: token_data.claims.role,
            }),
            None => Err(AuthError::InvalidToken), // User deleted
        }
    }
}

// Ensure Admin Middleware (Extractor)
pub struct AdminUser(pub AuthUser);

// #[async_trait]
impl FromRequestParts<AppState> for AdminUser {
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        let user = AuthUser::from_request_parts(parts, state).await?;
        
        if user.role == "admin" {
            Ok(AdminUser(user))
        } else {
            Err(AuthError::Forbidden)
        }
    }
}

#[derive(Debug)]
pub enum AuthError {
    MissingCredentials,
    InvalidToken,
    Forbidden,
    AccountDisabled,
    DatabaseError,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AuthError::MissingCredentials => (StatusCode::UNAUTHORIZED, "Missing credentials"),
            AuthError::InvalidToken => (StatusCode::UNAUTHORIZED, "Invalid token"),
            AuthError::Forbidden => (StatusCode::FORBIDDEN, "Access denied"),
            AuthError::AccountDisabled => (StatusCode::FORBIDDEN, "Account disabled"),
            AuthError::DatabaseError => (StatusCode::INTERNAL_SERVER_ERROR, "Database error"),
        };
        let body = Json(serde_json::json!({
            "error": error_message,
        }));
        (status, body).into_response()
    }
}
