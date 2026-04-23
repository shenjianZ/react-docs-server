use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct AuthConfig {
    #[serde(default = "default_jwt_secret")]
    pub jwt_secret: String,
    #[serde(default = "default_access_token_expiration_minutes")]
    pub access_token_expiration_minutes: u64,
    #[serde(default = "default_refresh_token_expiration_days")]
    pub refresh_token_expiration_days: i64,
    #[serde(default = "default_frontend_base_url")]
    pub frontend_base_url: String,
    #[serde(default)]
    pub email_verification: EmailVerificationConfig,
    #[serde(default)]
    pub providers: AuthProvidersConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct EmailVerificationConfig {
    #[serde(default = "default_email_code_ttl_seconds")]
    pub code_ttl_seconds: u64,
    #[serde(default = "default_email_send_interval_seconds")]
    pub send_interval_seconds: u64,
    #[serde(default = "default_email_subject_prefix")]
    pub subject_prefix: String,
    #[serde(default)]
    pub smtp: SmtpConfig,
}

impl Default for EmailVerificationConfig {
    fn default() -> Self {
        Self {
            code_ttl_seconds: default_email_code_ttl_seconds(),
            send_interval_seconds: default_email_send_interval_seconds(),
            subject_prefix: default_email_subject_prefix(),
            smtp: SmtpConfig::default(),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct SmtpConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_smtp_host")]
    pub host: String,
    #[serde(default = "default_smtp_port")]
    pub port: u16,
    #[serde(default)]
    pub username: String,
    #[serde(default)]
    pub password: String,
    #[serde(default = "default_smtp_from_email")]
    pub from_email: String,
    #[serde(default = "default_smtp_from_name")]
    pub from_name: String,
    #[serde(default = "default_smtp_starttls")]
    pub starttls: bool,
}

impl Default for SmtpConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            host: default_smtp_host(),
            port: default_smtp_port(),
            username: String::new(),
            password: String::new(),
            from_email: default_smtp_from_email(),
            from_name: default_smtp_from_name(),
            starttls: default_smtp_starttls(),
        }
    }
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct AuthProvidersConfig {
    #[serde(default)]
    pub google: OAuthProviderConfig,
    #[serde(default)]
    pub github: OAuthProviderConfig,
    #[serde(default)]
    pub wechat: OAuthProviderConfig,
    #[serde(default)]
    pub qq: OAuthProviderConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct OAuthProviderConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub client_id: String,
    #[serde(default)]
    pub client_secret: String,
    #[serde(default)]
    pub redirect_uri: String,
    #[serde(default)]
    pub scopes: Vec<String>,
}

impl Default for OAuthProviderConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            client_id: String::new(),
            client_secret: String::new(),
            redirect_uri: String::new(),
            scopes: Vec::new(),
        }
    }
}

fn default_jwt_secret() -> String {
    "change-this-to-a-strong-secret-key-in-production".to_string()
}

fn default_access_token_expiration_minutes() -> u64 {
    15
}

fn default_refresh_token_expiration_days() -> i64 {
    7
}

fn default_frontend_base_url() -> String {
    "http://localhost:5173".to_string()
}

fn default_email_code_ttl_seconds() -> u64 {
    300
}

fn default_email_send_interval_seconds() -> u64 {
    60
}

fn default_email_subject_prefix() -> String {
    "[React Docs]".to_string()
}

fn default_smtp_host() -> String {
    "smtp.example.com".to_string()
}

fn default_smtp_port() -> u16 {
    587
}

fn default_smtp_from_email() -> String {
    "noreply@example.com".to_string()
}

fn default_smtp_from_name() -> String {
    "React Docs".to_string()
}

fn default_smtp_starttls() -> bool {
    true
}
