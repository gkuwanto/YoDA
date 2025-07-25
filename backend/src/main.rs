use axum::{routing::{get, post, put, delete}, Router, http::StatusCode, response::IntoResponse, extract::Extension};
use sqlx::postgres::PgPoolOptions;
use dotenv::dotenv;
use std::env;
use std::sync::Arc;
mod models;
mod handlers;
mod middleware;
mod socket;
use middleware::{jwt_auth, AuthUser};
use socket::{SessionState, ws_handler};

async fn health_check() -> impl IntoResponse {
    (StatusCode::OK, "OK")
}

async fn protected_route(Extension(user): Extension<AuthUser>) -> impl IntoResponse {
    (StatusCode::OK, format!("User ID: {}", user.0))
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    // Set up PostgreSQL connection pool
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to Postgres");
    println!("Connected to Postgres");

    // Redis connection will be set up when needed for caching

    // Create shared session state for WebSocket connections
    let session_state = SessionState {
        sessions: Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
    };

    // Build our application with a health check route
    let app = Router::new()
        .route("/ws", get(ws_handler))
        .route("/auth/register", axum::routing::post(handlers::register))
        .route("/auth/login", axum::routing::post(handlers::login))
        .route("/protected", get(protected_route).route_layer(axum::middleware::from_fn(jwt_auth)))
        // Campaign routes (protected)
        .route("/campaigns", get(handlers::list_campaigns).route_layer(axum::middleware::from_fn(jwt_auth)))
        .route("/campaigns", post(handlers::create_campaign).route_layer(axum::middleware::from_fn(jwt_auth)))
        .route("/campaigns/:id", get(handlers::get_campaign).route_layer(axum::middleware::from_fn(jwt_auth)))
        .route("/campaigns/:id", put(handlers::update_campaign).route_layer(axum::middleware::from_fn(jwt_auth)))
        .route("/campaigns/:id", delete(handlers::delete_campaign).route_layer(axum::middleware::from_fn(jwt_auth)))
        // Session routes (protected)
        .route("/sessions", get(handlers::list_sessions).route_layer(axum::middleware::from_fn(jwt_auth)))
        .route("/sessions", post(handlers::create_session).route_layer(axum::middleware::from_fn(jwt_auth)))
        .route("/sessions/:id", get(handlers::get_session).route_layer(axum::middleware::from_fn(jwt_auth)))
        .route("/sessions/:id", put(handlers::update_session).route_layer(axum::middleware::from_fn(jwt_auth)))
        .route("/sessions/:id/start", post(handlers::start_session).route_layer(axum::middleware::from_fn(jwt_auth)))
        .route("/sessions/:id/end", post(handlers::end_session).route_layer(axum::middleware::from_fn(jwt_auth)))
        // Character routes (protected)
        .route("/characters", get(handlers::list_characters).route_layer(axum::middleware::from_fn(jwt_auth)))
        .route("/characters", post(handlers::create_character).route_layer(axum::middleware::from_fn(jwt_auth)))
        .route("/characters/:id", get(handlers::get_character).route_layer(axum::middleware::from_fn(jwt_auth)))
        .route("/characters/:id", put(handlers::update_character).route_layer(axum::middleware::from_fn(jwt_auth)))
        .route("/characters/:id", delete(handlers::delete_character).route_layer(axum::middleware::from_fn(jwt_auth)))
        .route("/characters/:id/hp", put(handlers::update_character_hp).route_layer(axum::middleware::from_fn(jwt_auth)))
        // Game state routes (protected)
        .route("/initiative", put(handlers::update_initiative).route_layer(axum::middleware::from_fn(jwt_auth)))
        // Health check endpoint
        .route("/health", get(health_check))
        .layer(Extension(pool))
        .layer(Extension(session_state));

    println!("🚀 YoDA Backend Server starting on http://0.0.0.0:3000");
    println!("📚 API Documentation available at http://localhost:3000/docs");
    println!("🔌 WebSocket endpoint available at ws://localhost:3000/ws");

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}