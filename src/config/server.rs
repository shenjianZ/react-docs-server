use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    #[serde(default = "default_server_host")]
    pub host: String,
    #[serde(default = "default_server_port")]
    pub port: u16,
    #[serde(default = "default_cors_allowed_origins")]
    pub cors_allowed_origins: Vec<String>,
}

fn default_server_host() -> String {
    "127.0.0.1".to_string()
}

fn default_server_port() -> u16 {
    3000
}

fn default_cors_allowed_origins() -> Vec<String> {
    vec!["http://localhost:5173".to_string()]
}
