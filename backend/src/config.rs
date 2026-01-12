use std::env;

#[derive(Clone, Debug)]
pub struct Config {
    pub database_url: String,
    pub server_host: String,
    pub server_port: u16,
    pub cors_allowed_origins: Vec<String>,
    pub max_wasm_size_mb: usize,
    pub solver_timeout_secs: u64,
    pub solver_instruction_limit: u64,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            database_url: env::var("DATABASE_URL")?,
            server_host: env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            server_port: env::var("SERVER_PORT")
                .unwrap_or_else(|_| "3000".to_string())
                .parse()?,
            cors_allowed_origins: env::var("CORS_ALLOWED_ORIGINS")
                .unwrap_or_else(|_| "http://localhost:5173".to_string())
                .split(',')
                .map(String::from)
                .collect(),
            max_wasm_size_mb: env::var("MAX_WASM_SIZE_MB")
                .unwrap_or_else(|_| "10".to_string())
                .parse()?,
            solver_timeout_secs: env::var("SOLVER_TIMEOUT_SECS")
                .unwrap_or_else(|_| "30".to_string())
                .parse()?,
            solver_instruction_limit: env::var("SOLVER_INSTRUCTION_LIMIT")
                .unwrap_or_else(|_| "10000000000".to_string())
                .parse()?,
        })
    }
}
