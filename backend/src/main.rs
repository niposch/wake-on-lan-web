mod db;

use sqlx::sqlite::SqlitePoolOptions;
use tower_http::services::ServeDir;
use axum::{Router, routing::get};

use crate::db::AppState;

use axum::{extract::State, http::StatusCode, Json};

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

#[tokio::main]
async fn main() {
    let api_routes = Router::new();

    let static_files = ServeDir::new("./static_files");

    let db_connection_string = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite:wol.db".to_string());

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&db_connection_string)
        .await
        .expect("Failed to connect to database");

    let state = AppState {
        db: pool
    };

    let app = Router::new()
        .nest("/api", api_routes)
        .route("/api/health", get(health_check))
        .fallback_service(static_files)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
