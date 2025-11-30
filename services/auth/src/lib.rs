pub mod auth_middleware;
pub mod config;
pub mod csrf;
pub mod database;
pub mod email_client;
pub mod handlers;
pub mod jwt;
pub mod models;
pub mod security;

pub use config::Config;
pub use database::Database;
pub use email_client::EmailClient;
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
    pub email_client: EmailClient,
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

    let email_client = EmailClient::new(
        config.email_service_url.clone(),
        config.email_service_api_key.clone(),
    );

    let state = Arc::new(AppState {
        db,
        jwt_service,
        email_client,
        config: config.clone(),
    });

    let cors = configure_cors(&config);

    let public_routes = Router::new()
        .route("/register", post(handlers::register))
        .route("/login", post(handlers::login))
        .route("/refresh", post(handlers::refresh))
        .route("/csrf-token", get(handlers::get_csrf_token))
        .route("/verify-email", post(handlers::verify_email))
        .route("/verify-otp", post(handlers::verify_email)) // Alias for frontend compatibility
        .route("/resend-verification", post(handlers::resend_verification))
        .route(
            "/request-password-reset",
            post(handlers::request_password_reset),
        )
        .route("/reset-password", post(handlers::reset_password))
        .with_state(state.clone());

    let protected_routes = Router::new()
        .route("/logout", post(handlers::logout))
        .route("/me", get(handlers::me))
        .route("/admin/check", get(handlers::admin_check))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware::auth_middleware,
        ))
        .with_state(state.clone());

    // Admin routes - require admin privileges
    let admin_routes = Router::new()
        .route("/admin/users", get(handlers::admin_list_users))
        .route("/admin/promote", post(handlers::admin_promote_user))
        .route("/admin/demote", post(handlers::admin_demote_user))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware::admin_middleware,
        ))
        .with_state(state.clone());

    let app = Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .merge(admin_routes)
        .route("/health", get(|| async { "OK" }))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            csrf::csrf_protection_middleware,
        ))
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
            .allow_headers([
                http::header::CONTENT_TYPE,
                http::header::AUTHORIZATION,
                http::HeaderName::from_static("x-csrf-token"),
            ])
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
