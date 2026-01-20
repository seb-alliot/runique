use serde::{Serialize, Deserialize};
use std::env;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
pub ip_server: String,
    pub domain_server: String,
    pub port: u16,
    pub secret_key: String,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            ip_server: "127.0.0.1".to_string(),
            port: 8080,
            secret_key: "change-me-in-production-very-secret".to_string(),
            domain_server: "localhost".to_string(),
        }
    }
}

impl ServerConfig {
    pub fn from_env() -> Self {
        let ip = env::var("IP_SERVER").unwrap_or_else(|_| "127.0.0.1".to_string());
        let port: u16 = env::var("PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(3000);

        ServerConfig {
            ip_server: ip.clone(),
            domain_server: format!("{}:{}", ip, port),
            port,
            secret_key: env::var("SECRET_KEY").unwrap_or_else(|_| "default_secret_key".to_string()),
        }
    }
}