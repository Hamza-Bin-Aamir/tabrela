pub mod auth_middleware;
pub mod config;
pub mod database;
pub mod handlers;
pub mod models;

pub use config::Config;
pub use database::Database;

use axum::{
    middleware,
    routing::{delete, get, post, put},
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

    // Public routes (optional authentication - show public data with optional user context)
    let public_routes = Router::new()
        // Match viewing (respects release toggles)
        .route("/matches/:match_id", get(handlers::get_match))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware::optional_auth_middleware,
        ))
        .with_state(state.clone());

    // Authenticated routes (require login)
    let authenticated_routes = Router::new()
        // Series viewing
        .route("/series", get(handlers::list_series))
        .route("/series/:series_id", get(handlers::get_series))
        // Match listing
        .route("/matches", get(handlers::list_matches))
        // Adjudicator ballot access
        .route("/matches/:match_id/my-ballot", get(handlers::get_my_ballot))
        .route(
            "/matches/:match_id/submit-ballot",
            post(handlers::submit_ballot),
        )
        .route(
            "/matches/:match_id/submit-feedback",
            post(handlers::submit_feedback),
        )
        // User performance
        .route(
            "/users/:user_id/performance",
            get(handlers::get_user_performance),
        )
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware::auth_middleware,
        ))
        .with_state(state.clone());

    // Admin routes - require admin privileges
    let admin_routes = Router::new()
        // Series management
        .route("/admin/series", post(handlers::create_series))
        .route("/admin/series/:series_id", put(handlers::update_series))
        .route("/admin/series/:series_id", delete(handlers::delete_series))
        // Match management
        .route("/admin/matches", post(handlers::create_match))
        .route("/admin/matches/:match_id", put(handlers::update_match))
        .route("/admin/matches/:match_id", delete(handlers::delete_match))
        .route(
            "/admin/matches/:match_id/release",
            post(handlers::toggle_release),
        )
        .route(
            "/admin/matches/:match_id/ballots",
            get(handlers::admin_get_match_ballots),
        )
        .route(
            "/admin/matches/:match_id/history",
            get(handlers::get_allocation_history),
        )
        // Team management
        .route("/admin/teams/:team_id", put(handlers::update_team))
        // Allocation management
        .route(
            "/admin/series/:series_id/pool",
            get(handlers::get_allocation_pool),
        )
        .route("/admin/allocations", post(handlers::create_allocation))
        .route(
            "/admin/allocations/:allocation_id",
            put(handlers::update_allocation),
        )
        .route(
            "/admin/allocations/:allocation_id",
            delete(handlers::delete_allocation),
        )
        .route("/admin/allocations/swap", post(handlers::swap_allocations))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware::admin_middleware,
        ))
        .with_state(state.clone());

    let app = Router::new()
        .merge(public_routes)
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
            .allow_headers([http::header::CONTENT_TYPE, http::header::AUTHORIZATION])
            .allow_credentials(true)
    }
}
