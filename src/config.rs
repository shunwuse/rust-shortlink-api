pub struct Config {
    pub port: u16,
    pub database_url: String,
    pub log_filter: String,
}

impl Config {
    pub fn from_env() -> Self {
        let port = std::env::var("PORT")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(3000);
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "sqlite:shortlink.db?mode=rwc".to_string());
        let log_filter = std::env::var("RUST_LOG")
            .unwrap_or_else(|_| "rust_shortlink_api=debug,tower_http=debug".to_string());

        Config {
            port,
            database_url,
            log_filter,
        }
    }
}
