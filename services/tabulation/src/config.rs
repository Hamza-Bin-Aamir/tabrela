use std::env;

#[derive(Clone, Debug)]
pub struct Config {
    pub database_url: String,
    pub host: String,
    pub port: u16,
    pub jwt_secret: String,
    pub auth_service_url: String,
    pub attendance_service_url: String,
    pub allowed_origins: Vec<String>,
    pub cors_strict_mode: bool,
}

impl Config {
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        // Try service-local .env first, then fall back to root .env
        dotenvy::dotenv().ok();
        dotenvy::from_filename("../../.env").ok();

        Ok(Config {
            database_url: env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgres://postgres:postgres@localhost/tabrela".to_string()),
            host: env::var("HOST")
                .or_else(|_| env::var("TABULATION_HOST"))
                .unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: env::var("PORT")
                .or_else(|_| env::var("TABULATION_PORT"))
                .unwrap_or_else(|_| "8084".to_string())
                .parse()?,
            jwt_secret: env::var("JWT_SECRET")
                .unwrap_or_else(|_| "your-super-secret-jwt-key-change-in-production".to_string()),
            auth_service_url: env::var("AUTH_SERVICE_URL")
                .unwrap_or_else(|_| "http://localhost:8081".to_string()),
            attendance_service_url: env::var("ATTENDANCE_SERVICE_URL")
                .unwrap_or_else(|_| "http://localhost:8082".to_string()),
            allowed_origins: env::var("ALLOWED_ORIGINS")
                .unwrap_or_else(|_| "*".to_string())
                .split(',')
                .map(|s| s.trim().to_string())
                .collect(),
            cors_strict_mode: env::var("CORS_STRICT_MODE")
                .unwrap_or_else(|_| "false".to_string())
                .parse()
                .unwrap_or(false),
        })
    }
}
