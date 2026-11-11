use std::env;

pub const PONDOK_LOGO_URL: &str = "/static/pondok-logo.svg";

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub host: String,
    pub port: u16,
    pub database_url: String,
    pub photo_storage_dir: String,
    pub max_photo_upload_bytes: usize,
    pub session_ttl_hours: i64,
    pub seed_on_startup: bool,
    pub wa_gateway_enabled: bool,
    pub wa_gateway_url: String,
    pub wa_gateway_api_key: String,
}

impl AppConfig {
    pub fn from_env() -> Self {
        let _ = dotenvy::dotenv();
        Self {
            host: env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: env::var("PORT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(3000),
            database_url: env::var("DATABASE_URL")
                .unwrap_or_else(|_| "sqlite://./data/sipesantren.db?mode=rwc".to_string()),
            photo_storage_dir: env::var("PHOTO_STORAGE_DIR")
                .unwrap_or_else(|_| "./data/private_photos".to_string()),
            max_photo_upload_bytes: env::var("MAX_PHOTO_UPLOAD_BYTES")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(2 * 1024 * 1024),
            session_ttl_hours: env::var("SESSION_TTL_HOURS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(168),
            seed_on_startup: env::var("SEED_ON_STARTUP")
                .map(|v| v.eq_ignore_ascii_case("true") || v == "1")
                .unwrap_or(true),
            wa_gateway_enabled: env::var("WA_GATEWAY_ENABLED")
                .map(|v| v.eq_ignore_ascii_case("true") || v == "1")
                .unwrap_or(false),
            wa_gateway_url: env::var("WA_GATEWAY_URL").unwrap_or_default(),
            wa_gateway_api_key: env::var("WA_GATEWAY_API_KEY").unwrap_or_default(),
        }
    }
}
