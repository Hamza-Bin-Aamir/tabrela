pub mod auth_middleware;
pub mod config;
pub mod database;
pub mod handlers;
pub mod models;

pub use config::Config;
pub use database::Database;

use axum::{
    middleware,
    routing::{get, post},
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

    // Truly public routes (optional authentication - extracts user if logged in)
    let unauthenticated_routes = Router::new()
        // Public profiles - shareable with anyone
        .route("/users/:username", get(handlers::get_profile_by_username))
        .route(
            "/users/:username/awards",
            get(handlers::get_user_awards_public),
        )
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware::optional_auth_middleware,
        ))
        .with_state(state.clone());

    // Authenticated routes (require login)
    let authenticated_routes = Router::new()
        // Own merit routes
        .route("/merit/me", get(handlers::get_my_merit))
        .route("/merit/me/history", get(handlers::get_my_merit_history))
        // Own awards routes
        .route("/awards/me", get(handlers::get_my_awards))
        .route("/awards/me/history", get(handlers::get_my_awards_history))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware::auth_middleware,
        ))
        .with_state(state.clone());

    // Admin routes - require admin privileges
    let admin_routes = Router::new()
        .route("/admin/merit", get(handlers::admin_list_all_merits))
        .route("/admin/merit", post(handlers::admin_update_merit))
        .route("/admin/merit/:user_id", get(handlers::admin_get_user_merit))
        .route(
            "/admin/merit/:user_id/history",
            get(handlers::admin_get_user_merit_history),
        )
        // Admin awards routes
        .route("/admin/awards", get(handlers::admin_list_all_awards))
        .route("/admin/awards", post(handlers::admin_create_award))
        .route(
            "/admin/awards/:award_id",
            axum::routing::put(handlers::admin_edit_award),
        )
        .route(
            "/admin/awards/:award_id/upgrade",
            axum::routing::patch(handlers::admin_upgrade_award),
        )
        .route(
            "/admin/awards/:user_id/history",
            get(handlers::admin_get_award_history),
        )
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware::admin_middleware,
        ))
        .with_state(state.clone());

    let app = Router::new()
        .merge(unauthenticated_routes)
        .merge(authenticated_routes)
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
        // Development mode - allow all origins with all methods and headers
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
            .allow_credentials(false) // Cannot use credentials with wildcard origin
    } else {
        // Specific origins mode (non-strict) - allow listed origins only
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
    }
}
