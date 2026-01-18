mod db;
mod api;

use sqlx::sqlite::SqlitePoolOptions;
use tower_http::services::ServeDir;
use axum::{Router, routing::{get, post, put, delete}};
use api::users;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::{api::users::UserApi, db::AppState};

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

#[derive(OpenApi)]
#[openapi(
    // We leave 'paths' empty here because we are merging modules below
    paths(), 
    tags(
        (name = "wol-app", description = "Wake-on-LAN API")
    )
)]
struct ApiDoc;

#[tokio::main]
async fn main() {
    let api_routes = Router::new()
        .route("/login", post(users::login))
        .route("/users", get(users::list_users).post(users::create_user))
        .route("/users/{id}", delete(users::delete_user))
        .route("/users/{id}/role", put(users::update_role))
        .route("/users/{id}/reset-password", post(users::admin_reset_password));

    // MERGE the module docs here
    let mut doc = ApiDoc::openapi();
    doc.merge(UserApi::openapi()); // <--- This pulls in all User paths & components


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
        .merge(SwaggerUi::new("/swagger").url("/api/openapi.json", doc.into()))
        .nest("/api", api_routes)
        .route("/api/health", get(health_check))
        .fallback_service(static_files)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
