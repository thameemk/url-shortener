use std::env;

#[derive(Clone)]
pub struct Config {
    pub database_url: String,
    pub api_rate_limit: u32,
    pub global_rate_limit: u32,
    pub port: u16,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            database_url: env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
            api_rate_limit: env::var("API_RATE_LIMIT")
                .expect("API_RATE_LIMIT must be set")
                .parse()
                .expect("API_RATE_LIMIT must be a valid number"),
            global_rate_limit: env::var("GLOBAL_RATE_LIMIT")
                .expect("GLOBAL_RATE_LIMIT must be set")
                .parse()
                .expect("GLOBAL_RATE_LIMIT must be a valid number"),
            port: env::var("PORT")
                .unwrap_or_else(|_| "8000".to_string())
                .parse()
                .expect("PORT must be a valid number"),
        }
    }
}
