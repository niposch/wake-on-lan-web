mod db;
mod api;
mod auth;

use sqlx::sqlite::SqlitePoolOptions;
use tower_http::services::ServeDir;
use axum::{Router, routing::{get, post, put, delete}};
use api::{users, devices};
use utoipa::{OpenApi, Modify};
use utoipa::openapi::security::{SecurityScheme, HttpAuthScheme, Http};
use utoipa_swagger_ui::SwaggerUi;
use clap::Parser;
use std::time::Duration;
use surge_ping::ping;
use std::net::IpAddr;

use crate::{api::users::UserApi, api::devices::DeviceApi, db::AppState};

use axum::{extract::State, http::StatusCode, Json};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Sets the initial admin password
    #[arg(long)]
    admin_password: Option<String>,
}

pub async fn health_check(
    State(state): State<AppState>,
) -> Result<Json<&'static str>, StatusCode> {
    // Basic DB check: simple query to verify DB is reachable
    if let Err(_) = sqlx::query("SELECT 1").execute(&state.db).await {
        Err(StatusCode::SERVICE_UNAVAILABLE)
    } else {
        Ok(Json("ok"))
    }
}

#[derive(OpenApi)]
#[openapi(
    // We leave 'paths' empty here because we are merging modules below
    paths(), 
    tags(
        (name = "wol-app", description = "Wake-on-LAN API")
    ),
    modifiers(&SecurityAddon),
    security(
        ("jwt" = [])
    )
)]
struct ApiDoc;

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "jwt",
                SecurityScheme::Http(
                    Http::new(HttpAuthScheme::Bearer)
                )
            )
        }
    }
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let db_connection_string = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite:wol.db".to_string());

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&db_connection_string)
        .await
        .expect("Failed to connect to database");

    // Initialize admin user if requested
    if let Some(password) = args.admin_password {
        println!("Initializing admin user...");
        let password_hash = users::hash_password(&password).expect("Failed to hash password");
        
        // Upsert admin user
        let result = sqlx::query!(
            r#"
            INSERT INTO users (username, password_hash, role, force_password_change)
            VALUES ('admin', ?, 'admin', 1)
            ON CONFLICT(username) DO UPDATE SET
                password_hash = excluded.password_hash,
                role = 'admin',
                force_password_change = 1
            "#,
            password_hash
        )
        .execute(&pool)
        .await;

        match result {
            Ok(_) => println!("Admin user initialized successfully with temporary password."),
            Err(e) => eprintln!("Failed to initialize admin user: {}", e),
        }
    }

    let pinger_pool = pool.clone();
    tokio::spawn(async move {
        loop {
            // Fetch all devices with IP addresses
            if let Ok(devices) = sqlx::query!("SELECT id, ip_address FROM devices WHERE ip_address IS NOT NULL")
                .fetch_all(&pinger_pool)
                .await 
            {
                for device in devices {
                    if let Some(ip_str) = device.ip_address {
                        if let Ok(ip) = ip_str.parse::<IpAddr>() {
                             // Ping with 1 second timeout
                             let is_online = match ping(ip, &[0; 8]).await {
                                 Ok((_, duration)) => {
                                     println!("Ping success for {}: {:?}", ip, duration);
                                     true
                                 },
                                 Err(_) => false,
                             };

                             let _ = sqlx::query!(
                                 "UPDATE devices SET is_online = ?, last_seen_at = CASE WHEN ? THEN CURRENT_TIMESTAMP ELSE last_seen_at END WHERE id = ?",
                                 is_online,
                                 is_online,
                                 device.id
                             )
                             .execute(&pinger_pool)
                             .await;
                        }
                    }
                }
            }
            
            tokio::time::sleep(Duration::from_secs(60)).await;
        }
    });

    let api_routes = Router::new()
        .route("/login", post(users::login))
        .route("/refresh", post(users::refresh_token))
        .route("/logout", post(users::logout_user))
        .route("/users", get(users::list_users).post(users::create_user))
        .route("/users/{id}", delete(users::delete_user))
        .route("/users/{id}/role", put(users::update_role))
        .route("/users/{id}/status", put(users::update_status))
        .route("/users/{id}/reset-password", post(users::admin_reset_password))
        .route("/change-password", post(users::change_password))
        .route("/me", get(users::get_me))
        // Devices
        .route("/devices", get(devices::list_devices).post(devices::create_device))
        .route("/devices/{id}", delete(devices::delete_device).put(devices::update_device))
        .route("/devices/{id}/wake", post(devices::wake_device))
        .route("/devices/{id}/shutdown", post(devices::shutdown_device));

    // MERGE the module docs here
    let mut doc = ApiDoc::openapi();
    doc.merge(UserApi::openapi()); // <--- This pulls in all User paths & components
    doc.merge(DeviceApi::openapi());


    let static_files = ServeDir::new("./static_files");


    let state = AppState {
        db: pool
    };

    let app = Router::new()
        .merge(SwaggerUi::new("/swagger").url("/api/openapi.json", doc.into()))
        .nest("/api", api_routes)
        .route("/api/health", get(health_check))
        .fallback_service(static_files)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
