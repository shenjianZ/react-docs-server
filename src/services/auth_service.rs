use anyhow::Result;
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use rand::{distributions::Alphanumeric, Rng};
use reqwest::header::{ACCEPT, AUTHORIZATION, USER_AGENT};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::str::FromStr;
use url::Url;
use uuid::Uuid;

use crate::config::auth::{AuthConfig, OAuthProviderConfig};
use crate::domain::dto::auth::{
    DeleteUserRequest, EmailCodeLoginRequest, EmailCodePurpose, LoginRequest, RegisterRequest,
    SendEmailCodeRequest,
};
use crate::domain::entities::users;
use crate::repositories::{
    oauth_account_repository::OAuthAccountRepository, user_repository::UserRepository,
};
use crate::services::email_service::EmailService;
use crate::utils::jwt::TokenService;
use crate::infra::redis::{
    redis_client::RedisClient,
    redis_key::{BusinessType, RedisKey},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OAuthProvider {
    Google,
    Github,
    Wechat,
    Qq,
}

impl OAuthProvider {
    pub fn as_str(&self) -> &'static str {
        match self {
            OAuthProvider::Google => "google",
            OAuthProvider::Github => "github",
            OAuthProvider::Wechat => "wechat",
            OAuthProvider::Qq => "qq",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            OAuthProvider::Google => "Google",
            OAuthProvider::Github => "GitHub",
            OAuthProvider::Wechat => "微信",
            OAuthProvider::Qq => "QQ",
        }
    }

    pub fn is_placeholder(&self) -> bool {
        matches!(self, OAuthProvider::Wechat | OAuthProvider::Qq)
    }
}

impl FromStr for OAuthProvider {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self> {
        match value {
            "google" => Ok(OAuthProvider::Google),
            "github" => Ok(OAuthProvider::Github),
            "wechat" => Ok(OAuthProvider::Wechat),
            "qq" => Ok(OAuthProvider::Qq),
            _ => Err(anyhow::anyhow!("不支持的第三方登录渠道")),
        }
    }
}

#[derive(Debug)]
pub struct OAuthStartResult {
    pub authorization_url: String,
}

#[derive(Debug)]
pub struct OAuthUserIdentity {
    pub provider_user_id: String,
    pub email: String,
    pub email_verified: bool,
    pub nickname: Option<String>,
    pub avatar_url: Option<String>,
    pub bio: Option<String>,
}

pub struct AuthService {
    user_repo: UserRepository,
    oauth_account_repo: OAuthAccountRepository,
    redis_client: RedisClient,
    auth_config: AuthConfig,
    email_service: EmailService,
}

impl AuthService {
    pub fn new(
        user_repo: UserRepository,
        oauth_account_repo: OAuthAccountRepository,
        redis_client: RedisClient,
        auth_config: AuthConfig,
    ) -> Self {
        let email_service = EmailService::new(auth_config.email_verification.smtp.clone());
        Self {
            user_repo,
            oauth_account_repo,
            redis_client,
            auth_config,
            email_service,
        }
    }

    pub fn hash_password(&self, password: &str) -> Result<String> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| anyhow::anyhow!("密码哈希失败: {}", e))?
            .to_string();
        Ok(password_hash)
    }

    pub fn generate_user_id(&self) -> String {
        let mut rng = rand::thread_rng();
        rng.gen_range(1_000_000_000i64..10_000_000_000i64)
            .to_string()
    }

    pub async fn generate_unique_user_id(&self) -> Result<String> {
        let mut attempts = 0;
        const MAX_ATTEMPTS: u32 = 10;

        loop {
            let candidate_id = self.generate_user_id();
            let existing = self.user_repo.count_by_id(&candidate_id).await?;
            if existing == 0 {
                return Ok(candidate_id);
            }

            attempts += 1;
            if attempts >= MAX_ATTEMPTS {
                return Err(anyhow::anyhow!("生成唯一用户 ID 失败"));
            }
        }
    }

    fn normalize_email(email: &str) -> String {
        email.trim().to_lowercase()
    }

    fn email_code_key(email: &str, purpose: EmailCodePurpose) -> String {
        RedisKey::new(BusinessType::Auth)
            .add_identifier("email_code")
            .add_identifier(purpose.as_str())
            .add_identifier(email)
            .build()
    }

    fn email_send_interval_key(email: &str, purpose: EmailCodePurpose) -> String {
        RedisKey::new(BusinessType::Auth)
            .add_identifier("email_code_send")
            .add_identifier(purpose.as_str())
            .add_identifier(email)
            .build()
    }

    fn email_code_attempt_key(email: &str, purpose: EmailCodePurpose) -> String {
        RedisKey::new(BusinessType::Auth)
            .add_identifier("email_code_attempt")
            .add_identifier(purpose.as_str())
            .add_identifier(email)
            .build()
    }

    fn oauth_state_key(provider: OAuthProvider, state: &str) -> String {
        RedisKey::new(BusinessType::Auth)
            .add_identifier("oauth_state")
            .add_identifier(provider.as_str())
            .add_identifier(state)
            .build()
    }

    fn hash_verification_code(email: &str, code: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(email.as_bytes());
        hasher.update(b":");
        hasher.update(code.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    fn generate_verification_code(&self) -> String {
        let mut rng = rand::thread_rng();
        format!("{:06}", rng.gen_range(0..1_000_000))
    }

    fn generate_state(&self) -> String {
        rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(32)
            .map(char::from)
            .collect()
    }

    fn provider_config(&self, provider: OAuthProvider) -> &OAuthProviderConfig {
        match provider {
            OAuthProvider::Google => &self.auth_config.providers.google,
            OAuthProvider::Github => &self.auth_config.providers.github,
            OAuthProvider::Wechat => &self.auth_config.providers.wechat,
            OAuthProvider::Qq => &self.auth_config.providers.qq,
        }
    }

    async fn save_refresh_token(
        &self,
        user_id: &str,
        refresh_token: &str,
        expiration_days: i64,
    ) -> Result<()> {
        let key = RedisKey::new(BusinessType::Auth)
            .add_identifier("refresh_token")
            .add_identifier(user_id);

        let expiration_seconds = expiration_days * 24 * 3600;

        self.redis_client
            .set_ex(&key.build(), refresh_token, expiration_seconds as u64)
            .await
            .map_err(|e| anyhow::anyhow!("Redis 保存失败: {}", e))?;

        Ok(())
    }

    async fn get_and_delete_refresh_token(&self, user_id: &str) -> Result<String> {
        let key = RedisKey::new(BusinessType::Auth)
            .add_identifier("refresh_token")
            .add_identifier(user_id);

        let token: Option<String> = self
            .redis_client
            .get(&key.build())
            .await
            .map_err(|e| anyhow::anyhow!("Redis 查询失败: {}", e))?;

        if token.is_some() {
            self.redis_client
                .delete_key(&key)
                .await
                .map_err(|e| anyhow::anyhow!("Redis 删除失败: {}", e))?;
        }

        token.ok_or_else(|| anyhow::anyhow!("刷新令牌无效或已过期"))
    }

    async fn issue_tokens_for_user(&self, user: users::Model) -> Result<(users::Model, String, String)> {
        let (access_token, refresh_token) = TokenService::generate_token_pair(
            &user.id,
            self.auth_config.access_token_expiration_minutes,
            self.auth_config.refresh_token_expiration_days,
            &self.auth_config.jwt_secret,
        )?;

        self.save_refresh_token(
            &user.id,
            &refresh_token,
            self.auth_config.refresh_token_expiration_days,
        )
        .await?;

        let user = self.user_repo.update_last_login(&user.id).await?;
        Ok((user, access_token, refresh_token))
    }

    async fn consume_email_code(
        &self,
        email: &str,
        purpose: EmailCodePurpose,
        verification_code: &str,
    ) -> Result<()> {
        let key = Self::email_code_key(email, purpose);
        let attempt_key = Self::email_code_attempt_key(email, purpose);
        let stored_hash = self
            .redis_client
            .get(&key)
            .await
            .map_err(|e| anyhow::anyhow!("验证码读取失败: {}", e))?
            .ok_or_else(|| anyhow::anyhow!("验证码不存在或已过期"))?;

        let input_hash = Self::hash_verification_code(email, verification_code.trim());
        if stored_hash != input_hash {
            let attempts = self
                .redis_client
                .incr(&attempt_key)
                .await
                .map_err(|e| anyhow::anyhow!("验证码校验失败: {}", e))?;
            if attempts == 1 {
                self.redis_client
                    .expire(&attempt_key, self.auth_config.email_verification.code_ttl_seconds)
                    .await
                    .map_err(|e| anyhow::anyhow!("验证码校验失败: {}", e))?;
            }
            if attempts >= 5 {
                let _ = self.redis_client.del(&key).await;
                let _ = self.redis_client.del(&attempt_key).await;
                return Err(anyhow::anyhow!("验证码错误次数过多，请重新获取"));
            }
            return Err(anyhow::anyhow!("验证码错误"));
        }

        self.redis_client
            .del(&key)
            .await
            .map_err(|e| anyhow::anyhow!("验证码销毁失败: {}", e))?;
        let _ = self.redis_client.del(&attempt_key).await;

        Ok(())
    }

    fn require_valid_email(email: &str) -> Result<String> {
        let normalized = Self::normalize_email(email);
        if normalized.is_empty() || !normalized.contains('@') {
            return Err(anyhow::anyhow!("请输入有效的邮箱地址"));
        }
        Ok(normalized)
    }

    pub async fn send_email_code(&self, request: SendEmailCodeRequest) -> Result<()> {
        let email = Self::require_valid_email(&request.email)?;
        match request.purpose {
            EmailCodePurpose::Register => {
                if self.user_repo.count_by_email(&email).await? > 0 {
                    return Err(anyhow::anyhow!("邮箱已注册"));
                }
            }
            EmailCodePurpose::Login => {
                if self.user_repo.count_by_email(&email).await? == 0 {
                    return Err(anyhow::anyhow!("该邮箱尚未注册"));
                }
            }
        }

        let interval_key = Self::email_send_interval_key(&email, request.purpose);
        if self
            .redis_client
            .get(&interval_key)
            .await
            .map_err(|e| anyhow::anyhow!("发送频率校验失败: {}", e))?
            .is_some()
        {
            return Err(anyhow::anyhow!("请求过于频繁，请稍后再试"));
        }

        let code = self.generate_verification_code();
        let code_hash = Self::hash_verification_code(&email, &code);
        let code_key = Self::email_code_key(&email, request.purpose);
        self.redis_client
            .set_ex(
                &code_key,
                &code_hash,
                self.auth_config.email_verification.code_ttl_seconds,
            )
            .await
            .map_err(|e| anyhow::anyhow!("验证码保存失败: {}", e))?;
        self.redis_client
            .set_ex(
                &interval_key,
                "1",
                self.auth_config.email_verification.send_interval_seconds,
            )
            .await
            .map_err(|e| anyhow::anyhow!("验证码限流失败: {}", e))?;
        let _ = self
            .redis_client
            .del(&Self::email_code_attempt_key(&email, request.purpose))
            .await;

        let purpose_label = match request.purpose {
            EmailCodePurpose::Register => "注册",
            EmailCodePurpose::Login => "登录",
        };
        if let Err(e) = self
            .email_service
            .send_verification_code(
                &email,
                &code,
                purpose_label,
                self.auth_config.email_verification.code_ttl_seconds,
                &self.auth_config.email_verification.subject_prefix,
            )
            .await
        {
            let _ = self.redis_client.del(&code_key).await;
            let _ = self.redis_client.del(&interval_key).await;
            return Err(e);
        }

        Ok(())
    }

    pub async fn register(
        &self,
        request: RegisterRequest,
    ) -> Result<(users::Model, String, String)> {
        let email = Self::require_valid_email(&request.email)?;
        if request.password.trim().len() < 6 {
            return Err(anyhow::anyhow!("密码至少 6 位"));
        }

        let existing = self.user_repo.count_by_email(&email).await?;
        if existing > 0 {
            return Err(anyhow::anyhow!("邮箱已注册"));
        }

        self.consume_email_code(
            &email,
            EmailCodePurpose::Register,
            &request.verification_code,
        )
        .await?;

        let password_hash = self.hash_password(&request.password)?;
        let user_id = self.generate_unique_user_id().await?;
        let user = self
            .user_repo
            .insert_local(user_id, email, password_hash, true)
            .await?;

        self.issue_tokens_for_user(user).await
    }

    pub async fn login(&self, request: LoginRequest) -> Result<(users::Model, String, String)> {
        let email = Self::require_valid_email(&request.email)?;
        let user = self
            .user_repo
            .find_by_email(&email)
            .await?
            .ok_or_else(|| anyhow::anyhow!("邮箱或密码错误"))?;

        if !user.password_set {
            return Err(anyhow::anyhow!(
                "当前账号未设置本地密码，请使用邮箱验证码或第三方登录"
            ));
        }

        let password_hash = self
            .user_repo
            .get_password_hash(&email)
            .await?
            .ok_or_else(|| anyhow::anyhow!("邮箱或密码错误"))?;

        let parsed_hash = PasswordHash::new(&password_hash)
            .map_err(|e| anyhow::anyhow!("解析密码哈希失败: {}", e))?;
        let argon2 = Argon2::default();
        argon2
            .verify_password(request.password.as_bytes(), &parsed_hash)
            .map_err(|_| anyhow::anyhow!("邮箱或密码错误"))?;

        self.issue_tokens_for_user(user).await
    }

    pub async fn login_with_email_code(
        &self,
        request: EmailCodeLoginRequest,
    ) -> Result<(users::Model, String, String)> {
        let email = Self::require_valid_email(&request.email)?;
        self.consume_email_code(&email, EmailCodePurpose::Login, &request.verification_code)
            .await?;

        let user = self
            .user_repo
            .find_by_email(&email)
            .await?
            .ok_or_else(|| anyhow::anyhow!("该邮箱尚未注册"))?;

        let user = if user.email_verified {
            user
        } else {
            self.user_repo.update_email_verified(&user.id, true).await?
        };

        self.issue_tokens_for_user(user).await
    }

    pub async fn build_oauth_authorization_url(
        &self,
        provider: OAuthProvider,
    ) -> Result<OAuthStartResult> {
        if provider.is_placeholder() {
            return Err(anyhow::anyhow!("{}登录暂未开放", provider.display_name()));
        }

        let config = self.provider_config(provider);
        if !config.enabled {
            return Err(anyhow::anyhow!("{}登录未启用", provider.display_name()));
        }
        if config.client_id.is_empty() || config.client_secret.is_empty() || config.redirect_uri.is_empty() {
            return Err(anyhow::anyhow!("{}登录配置不完整", provider.display_name()));
        }

        let state = self.generate_state();
        let state_key = Self::oauth_state_key(provider, &state);
        self.redis_client
            .set_ex(&state_key, "1", 600)
            .await
            .map_err(|e| anyhow::anyhow!("OAuth 状态保存失败: {}", e))?;

        let scopes = if config.scopes.is_empty() {
            match provider {
                OAuthProvider::Google => vec!["openid".to_string(), "email".to_string(), "profile".to_string()],
                OAuthProvider::Github => vec!["read:user".to_string(), "user:email".to_string()],
                _ => Vec::new(),
            }
        } else {
            config.scopes.clone()
        };

        let mut url = match provider {
            OAuthProvider::Google => Url::parse("https://accounts.google.com/o/oauth2/v2/auth")?,
            OAuthProvider::Github => Url::parse("https://github.com/login/oauth/authorize")?,
            _ => unreachable!(),
        };

        {
            let mut pairs = url.query_pairs_mut();
            pairs.append_pair("client_id", &config.client_id);
            pairs.append_pair("redirect_uri", &config.redirect_uri);
            pairs.append_pair("state", &state);
            pairs.append_pair("scope", &scopes.join(" "));
            match provider {
                OAuthProvider::Google => {
                    pairs.append_pair("response_type", "code");
                    pairs.append_pair("access_type", "online");
                    pairs.append_pair("prompt", "select_account");
                }
                OAuthProvider::Github => {}
                _ => {}
            }
        }

        Ok(OAuthStartResult {
            authorization_url: url.to_string(),
        })
    }

    async fn consume_oauth_state(&self, provider: OAuthProvider, state: &str) -> Result<()> {
        let key = Self::oauth_state_key(provider, state);
        let stored = self
            .redis_client
            .get(&key)
            .await
            .map_err(|e| anyhow::anyhow!("OAuth 状态读取失败: {}", e))?;
        if stored.is_none() {
            return Err(anyhow::anyhow!("OAuth 状态无效或已过期"));
        }
        self.redis_client
            .del(&key)
            .await
            .map_err(|e| anyhow::anyhow!("OAuth 状态销毁失败: {}", e))?;
        Ok(())
    }

    async fn fetch_oauth_identity(
        &self,
        provider: OAuthProvider,
        code: &str,
    ) -> Result<OAuthUserIdentity> {
        match provider {
            OAuthProvider::Google => self.fetch_google_identity(code).await,
            OAuthProvider::Github => self.fetch_github_identity(code).await,
            OAuthProvider::Wechat | OAuthProvider::Qq => {
                Err(anyhow::anyhow!("{}登录暂未开放", provider.display_name()))
            }
        }
    }

    async fn fetch_google_identity(&self, code: &str) -> Result<OAuthUserIdentity> {
        #[derive(Deserialize)]
        struct GoogleTokenResponse {
            access_token: String,
        }

        #[derive(Deserialize)]
        struct GoogleUserResponse {
            sub: String,
            email: Option<String>,
            email_verified: Option<bool>,
            name: Option<String>,
            picture: Option<String>,
        }

        let config = self.provider_config(OAuthProvider::Google);
        let client = reqwest::Client::new();
        let token = client
            .post("https://oauth2.googleapis.com/token")
            .form(&[
                ("client_id", config.client_id.as_str()),
                ("client_secret", config.client_secret.as_str()),
                ("code", code),
                ("grant_type", "authorization_code"),
                ("redirect_uri", config.redirect_uri.as_str()),
            ])
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Google token 请求失败: {}", e))?
            .error_for_status()
            .map_err(|e| anyhow::anyhow!("Google token 获取失败: {}", e))?
            .json::<GoogleTokenResponse>()
            .await
            .map_err(|e| anyhow::anyhow!("解析 Google token 响应失败: {}", e))?;

        let profile = client
            .get("https://openidconnect.googleapis.com/v1/userinfo")
            .header(AUTHORIZATION, format!("Bearer {}", token.access_token))
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Google 用户信息请求失败: {}", e))?
            .error_for_status()
            .map_err(|e| anyhow::anyhow!("Google 用户信息获取失败: {}", e))?
            .json::<GoogleUserResponse>()
            .await
            .map_err(|e| anyhow::anyhow!("解析 Google 用户信息失败: {}", e))?;

        let email = profile
            .email
            .map(|email| Self::normalize_email(&email))
            .ok_or_else(|| anyhow::anyhow!("Google 未返回可用邮箱"))?;

        Ok(OAuthUserIdentity {
            provider_user_id: profile.sub,
            email,
            email_verified: profile.email_verified.unwrap_or(false),
            nickname: profile.name,
            avatar_url: profile.picture,
            bio: Some("使用 Google 登录创建的账号".to_string()),
        })
    }

    async fn fetch_github_identity(&self, code: &str) -> Result<OAuthUserIdentity> {
        #[derive(Deserialize)]
        struct GithubTokenResponse {
            access_token: String,
        }

        #[derive(Deserialize)]
        struct GithubUserResponse {
            id: u64,
            login: String,
            name: Option<String>,
            avatar_url: Option<String>,
            bio: Option<String>,
            email: Option<String>,
        }

        #[derive(Deserialize)]
        struct GithubEmailResponse {
            email: String,
            primary: bool,
            verified: bool,
        }

        let config = self.provider_config(OAuthProvider::Github);
        let client = reqwest::Client::new();
        let token = client
            .post("https://github.com/login/oauth/access_token")
            .header(ACCEPT, "application/json")
            .header(USER_AGENT, "react-docs-server")
            .form(&[
                ("client_id", config.client_id.as_str()),
                ("client_secret", config.client_secret.as_str()),
                ("code", code),
                ("redirect_uri", config.redirect_uri.as_str()),
            ])
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("GitHub token 请求失败: {}", e))?
            .error_for_status()
            .map_err(|e| anyhow::anyhow!("GitHub token 获取失败: {}", e))?
            .json::<GithubTokenResponse>()
            .await
            .map_err(|e| anyhow::anyhow!("解析 GitHub token 响应失败: {}", e))?;

        let user = client
            .get("https://api.github.com/user")
            .header(AUTHORIZATION, format!("Bearer {}", token.access_token))
            .header(USER_AGENT, "react-docs-server")
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("GitHub 用户信息请求失败: {}", e))?
            .error_for_status()
            .map_err(|e| anyhow::anyhow!("GitHub 用户信息获取失败: {}", e))?
            .json::<GithubUserResponse>()
            .await
            .map_err(|e| anyhow::anyhow!("解析 GitHub 用户信息失败: {}", e))?;

        let mut email = user.email.map(|value| Self::normalize_email(&value));
        let mut email_verified = email.is_some();

        if email.is_none() {
            let emails = client
                .get("https://api.github.com/user/emails")
                .header(AUTHORIZATION, format!("Bearer {}", token.access_token))
                .header(USER_AGENT, "react-docs-server")
                .send()
                .await
                .map_err(|e| anyhow::anyhow!("GitHub 邮箱列表请求失败: {}", e))?
                .error_for_status()
                .map_err(|e| anyhow::anyhow!("GitHub 邮箱列表获取失败: {}", e))?
                .json::<Vec<GithubEmailResponse>>()
                .await
                .map_err(|e| anyhow::anyhow!("解析 GitHub 邮箱列表失败: {}", e))?;

            if let Some(primary) = emails.into_iter().find(|item| item.primary && item.verified) {
                email = Some(Self::normalize_email(&primary.email));
                email_verified = true;
            }
        }

        let email = email.ok_or_else(|| anyhow::anyhow!("GitHub 未返回可用邮箱"))?;

        Ok(OAuthUserIdentity {
            provider_user_id: user.id.to_string(),
            email,
            email_verified,
            nickname: user.name.or(Some(user.login)),
            avatar_url: user.avatar_url,
            bio: user.bio,
        })
    }

    async fn find_or_create_oauth_user(
        &self,
        provider: OAuthProvider,
        identity: OAuthUserIdentity,
    ) -> Result<users::Model> {
        if !identity.email_verified {
            return Err(anyhow::anyhow!("第三方登录未返回已验证邮箱，无法完成登录"));
        }

        if let Some(account) = self
            .oauth_account_repo
            .find_by_provider_user_id(provider.as_str(), &identity.provider_user_id)
            .await?
        {
            let user = self
                .user_repo
                .find_by_id(&account.user_id)
                .await?
                .ok_or_else(|| anyhow::anyhow!("关联用户不存在"))?;
            return Ok(user);
        }

        let user = if let Some(existing_user) = self.user_repo.find_by_email(&identity.email).await? {
            if existing_user.email_verified {
                existing_user
            } else {
                self.user_repo
                    .update_email_verified(&existing_user.id, true)
                    .await?
            }
        } else {
            let user_id = self.generate_unique_user_id().await?;
            let password_hash = self.hash_password(&format!("oauth-only-{}", Uuid::new_v4()))?;
            self.user_repo
                .insert_oauth_user(
                    user_id,
                    identity.email.clone(),
                    password_hash,
                    true,
                    identity.nickname.clone(),
                    identity.avatar_url.clone(),
                    identity.bio.clone(),
                )
                .await?
        };

        if self
            .oauth_account_repo
            .find_by_user_id_and_provider(&user.id, provider.as_str())
            .await?
            .is_none()
        {
            self.oauth_account_repo
                .insert(
                    user.id.clone(),
                    provider.as_str().to_string(),
                    identity.provider_user_id,
                    Some(identity.email),
                )
                .await?;
        }

        Ok(user)
    }

    pub async fn login_with_oauth_callback(
        &self,
        provider: OAuthProvider,
        code: &str,
        state: &str,
    ) -> Result<(users::Model, String, String)> {
        self.consume_oauth_state(provider, state).await?;
        let identity = self.fetch_oauth_identity(provider, code).await?;
        let user = self.find_or_create_oauth_user(provider, identity).await?;
        self.issue_tokens_for_user(user).await
    }

    pub async fn delete_refresh_token(&self, user_id: &str) -> Result<()> {
        let key = RedisKey::new(BusinessType::Auth)
            .add_identifier("refresh_token")
            .add_identifier(user_id);

        self.redis_client
            .delete_key(&key)
            .await
            .map_err(|e| anyhow::anyhow!("Redis 删除失败: {}", e))?;

        Ok(())
    }

    pub async fn refresh_access_token(&self, refresh_token: &str) -> Result<(String, String)> {
        let user_id = TokenService::decode_user_id(refresh_token, &self.auth_config.jwt_secret)?;
        let stored_token = self.get_and_delete_refresh_token(&user_id).await?;
        if stored_token != refresh_token {
            return Err(anyhow::anyhow!("刷新令牌无效"));
        }

        let (new_access_token, new_refresh_token) = TokenService::generate_token_pair(
            &user_id,
            self.auth_config.access_token_expiration_minutes,
            self.auth_config.refresh_token_expiration_days,
            &self.auth_config.jwt_secret,
        )?;

        self.save_refresh_token(
            &user_id,
            &new_refresh_token,
            self.auth_config.refresh_token_expiration_days,
        )
        .await?;

        Ok((new_access_token, new_refresh_token))
    }

    pub async fn delete_user(&self, request: DeleteUserRequest) -> Result<()> {
        let user = self
            .user_repo
            .find_by_id(&request.user_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("用户不存在"))?;

        if !user.password_set {
            return Err(anyhow::anyhow!(
                "当前账号未设置本地密码，请先使用邮箱验证码或第三方登录后再完成账号处理"
            ));
        }

        let password_hash = self
            .user_repo
            .get_password_hash_by_user_id(&request.user_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("用户不存在"))?;

        let parsed_hash = PasswordHash::new(&password_hash)
            .map_err(|e| anyhow::anyhow!("解析密码哈希失败: {}", e))?;
        let argon2 = Argon2::default();

        argon2
            .verify_password(request.password.as_bytes(), &parsed_hash)
            .map_err(|_| anyhow::anyhow!("密码错误"))?;

        self.user_repo.delete_by_id(&request.user_id).await?;

        Ok(())
    }
}
