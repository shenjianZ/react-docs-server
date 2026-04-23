use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct RegisterResult {
    pub id: String,
    pub email: String,
    pub email_verified: bool,
    pub username: Option<String>,
    pub nickname: Option<String>,
    pub avatar_url: Option<String>,
    pub bio: Option<String>,
    pub role: Option<String>,
    pub status: Option<String>,
    pub created_at: String,
    pub access_token: String,
    pub refresh_token: String,
}

impl From<(crate::domain::entities::users::Model, String, String)> for RegisterResult {
    fn from(
        (user_model, access_token, refresh_token): (
            crate::domain::entities::users::Model,
            String,
            String,
        ),
    ) -> Self {
        Self {
            id: user_model.id,
            email: user_model.email,
            email_verified: user_model.email_verified,
            username: user_model.username,
            nickname: user_model.nickname,
            avatar_url: user_model.avatar_url,
            bio: user_model.bio,
            role: user_model.role,
            status: user_model.status,
            created_at: user_model
                .created_at
                .format("%Y-%m-%dT%H:%M:%S%.3fZ")
                .to_string(),
            access_token,
            refresh_token,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct LoginResult {
    pub id: String,
    pub email: String,
    pub email_verified: bool,
    pub username: Option<String>,
    pub nickname: Option<String>,
    pub avatar_url: Option<String>,
    pub bio: Option<String>,
    pub role: Option<String>,
    pub status: Option<String>,
    pub created_at: String,
    pub access_token: String,
    pub refresh_token: String,
}

impl From<(crate::domain::entities::users::Model, String, String)> for LoginResult {
    fn from(
        (user_model, access_token, refresh_token): (
            crate::domain::entities::users::Model,
            String,
            String,
        ),
    ) -> Self {
        Self {
            id: user_model.id,
            email: user_model.email,
            email_verified: user_model.email_verified,
            username: user_model.username,
            nickname: user_model.nickname,
            avatar_url: user_model.avatar_url,
            bio: user_model.bio,
            role: user_model.role,
            status: user_model.status,
            created_at: user_model
                .created_at
                .format("%Y-%m-%dT%H:%M:%S%.3fZ")
                .to_string(),
            access_token,
            refresh_token,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct RefreshResult {
    pub access_token: String,
    pub refresh_token: String,
}
