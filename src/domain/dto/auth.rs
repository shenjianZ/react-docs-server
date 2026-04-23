use serde::Deserialize;
use std::fmt;

/// 注册请求
#[derive(Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub verification_code: String,
}

impl fmt::Debug for RegisterRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "RegisterRequest {{ email: {}, password: ***, verification_code: *** }}",
            self.email
        )
    }
}

/// 登录请求
#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

impl fmt::Debug for LoginRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "LoginRequest {{ email: {}, password: *** }}", self.email)
    }
}

#[derive(Deserialize, Clone, Copy, Debug)]
#[serde(rename_all = "snake_case")]
pub enum EmailCodePurpose {
    Register,
    Login,
}

impl EmailCodePurpose {
    pub fn as_str(&self) -> &'static str {
        match self {
            EmailCodePurpose::Register => "register",
            EmailCodePurpose::Login => "login",
        }
    }
}

#[derive(Deserialize)]
pub struct SendEmailCodeRequest {
    pub email: String,
    pub purpose: EmailCodePurpose,
}

impl fmt::Debug for SendEmailCodeRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "SendEmailCodeRequest {{ email: {}, purpose: {} }}",
            self.email,
            self.purpose.as_str()
        )
    }
}

#[derive(Deserialize)]
pub struct EmailCodeLoginRequest {
    pub email: String,
    pub verification_code: String,
}

impl fmt::Debug for EmailCodeLoginRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "EmailCodeLoginRequest {{ email: {}, verification_code: *** }}",
            self.email
        )
    }
}

#[derive(Deserialize, Debug)]
pub struct OAuthCallbackQuery {
    pub code: Option<String>,
    pub state: Option<String>,
    pub error: Option<String>,
    pub error_description: Option<String>,
}

/// 删除用户请求
#[derive(Deserialize)]
pub struct DeleteUserRequest {
    pub user_id: String,
    pub password: String,
}

impl fmt::Debug for DeleteUserRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "DeleteUserRequest {{ user_id: {}, password: *** }}",
            self.user_id
        )
    }
}

/// 刷新令牌请求
#[derive(Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

impl fmt::Debug for RefreshRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "RefreshRequest {{ refresh_token: *** }}")
    }
}

#[derive(Deserialize, Debug)]
pub struct UpdateProfileRequest {
    pub username: Option<String>,
    pub nickname: Option<String>,
    pub bio: Option<String>,
}
