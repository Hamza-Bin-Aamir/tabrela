pub mod config;
pub mod csrf;
pub mod database;
pub mod handlers;
pub mod jwt;
pub mod models;
pub mod security;
pub mod auth_middleware;

pub use config::Config;
pub use database::Database;
pub use jwt::JwtService;

use axum::{
    middleware,
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};

pub struct AppState {
    pub db: Database,
    pub jwt_service: JwtService,
    pub config: Config,
}

pub async fn create_app() -> Result<Router, Box<dyn std::error::Error>> {
    let config = Config::from_env()?;
    let db = Database::new(&config.database_url).await?;
    db.migrate().await?;

    let jwt_service = JwtService::new(
        config.jwt_secret.clone(),
        config.jwt_access_token_expiry,
        config.jwt_refresh_token_expiry,
    );

    let state = Arc::new(AppState {
        db,
        jwt_service,
        config: config.clone(),
    });

    let cors = configure_cors(&config);

    let public_routes = Router::new()
        .route("/register", post(handlers::register))
        .route("/login", post(handlers::login))
        .route("/refresh", post(handlers::refresh))
        .route("/csrf-token", get(handlers::get_csrf_token))
        .with_state(state.clone());

    let protected_routes = Router::new()
        .route("/logout", post(handlers::logout))
        .route("/me", get(handlers::me))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware::auth_middleware,
        ))
        .with_state(state.clone());

    let app = Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .route("/health", get(|| async { "OK" }))
        .layer(cors)
        .layer(middleware::from_fn_with_state(
            state.clone(),
            csrf::csrf_protection_middleware,
        ));

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
            ])
            .allow_headers([
                http::header::CONTENT_TYPE,
                http::header::AUTHORIZATION,
                http::HeaderName::from_static("x-csrf-token"),
            ])
            .allow_credentials(true)
    } else {
        if config.allowed_origins.contains(&"*".to_string()) {
            CorsLayer::permissive()
        } else {
            let mut cors_layer = CorsLayer::new().allow_origin(Any);
            for origin in &config.allowed_origins {
                if origin != "*" {
                    if let Ok(header_value) = origin.parse::<http::header::HeaderValue>() {
                        cors_layer = cors_layer.allow_origin(header_value);
                    }
                }
            }
            cors_layer.allow_methods(Any).allow_headers(Any)
        }
    }
}
