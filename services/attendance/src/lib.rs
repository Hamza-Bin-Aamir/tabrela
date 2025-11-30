pub mod auth_middleware;
pub mod config;
pub mod database;
pub mod handlers;
pub mod models;

pub use config::Config;
pub use database::Database;

use axum::{
    middleware,
    routing::{delete, get, patch, post},
    Router,
};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};

pub struct AppState {
    pub db: Database,
    pub config: Config,
}

pub async fn create_app() -> Result<Router, Box<dyn std::error::Error>> {
    let config = Config::from_env()?;
    let db = Database::new(&config.database_url).await?;
    db.migrate().await?;

    let state = Arc::new(AppState {
        db,
        config: config.clone(),
    });

    let cors = configure_cors(&config);

    // Public routes (require authentication)
    let public_routes = Router::new()
        .route("/events", get(handlers::list_events))
        .route("/events/:event_id", get(handlers::get_event))
        .route(
            "/events/:event_id/attendance",
            get(handlers::get_event_attendance),
        )
        .route(
            "/events/:event_id/my-attendance",
            get(handlers::get_my_attendance),
        )
        .route(
            "/events/:event_id/availability",
            post(handlers::set_availability),
        )
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware::auth_middleware,
        ))
        .with_state(state.clone());

    // Admin routes
    let admin_routes = Router::new()
        .route("/events", post(handlers::create_event))
        .route("/events/:event_id", patch(handlers::update_event))
        .route("/events/:event_id", delete(handlers::delete_event))
        .route("/events/:event_id/lock", post(handlers::lock_event))
        .route("/events/:event_id/check-in", post(handlers::check_in_user))
        .route(
            "/events/:event_id/revoke",
            post(handlers::revoke_availability),
        )
        .route("/attendance/matrix", get(handlers::get_attendance_matrix))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware::admin_middleware,
        ))
        .with_state(state.clone());

    let app = Router::new()
        .merge(public_routes)
        .merge(admin_routes)
        .route("/health", get(|| async { "OK" }))
        .layer(cors);

    Ok(app)
}

fn configure_cors(config: &Config) -> CorsLayer {
    if config.cors_strict_mode {
        let mut cors_layer = CorsLayer::new();
        for origin in &config.allowed_origins {
            if origin != "*" {
                if let Ok(header_value) = origin.parse::<http::header::HeaderValue>() {
                    cors_layer = cors_layer.allow_origin(header_value);
                }
            }
        }
        cors_layer
            .allow_methods([
                http::Method::GET,
                http::Method::POST,
                http::Method::PUT,
                http::Method::PATCH,
                http::Method::DELETE,
                http::Method::OPTIONS,
            ])
            .allow_headers([http::header::CONTENT_TYPE, http::header::AUTHORIZATION])
            .allow_credentials(true)
    } else if config.allowed_origins.contains(&"*".to_string()) {
        CorsLayer::new()
            .allow_origin(Any)
            .allow_methods([
                http::Method::GET,
                http::Method::POST,
                http::Method::PUT,
                http::Method::PATCH,
                http::Method::DELETE,
                http::Method::OPTIONS,
            ])
            .allow_headers(Any)
            .allow_credentials(false)
    } else {
        let mut cors_layer = CorsLayer::new().allow_origin(Any);
        for origin in &config.allowed_origins {
            if origin != "*" {
                if let Ok(header_value) = origin.parse::<http::header::HeaderValue>() {
                    cors_layer = cors_layer.allow_origin(header_value);
                }
            }
        }
        cors_layer
            .allow_methods([
                http::Method::GET,
                http::Method::POST,
                http::Method::PUT,
                http::Method::PATCH,
                http::Method::DELETE,
                http::Method::OPTIONS,
            ])
            .allow_headers(Any)
    }
}
