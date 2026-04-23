use serde::Serialize;

/// 当前登录用户资料
#[derive(Debug, Serialize)]
pub struct UserProfile {
    pub id: String,
    pub email: String,
    pub username: Option<String>,
    pub nickname: Option<String>,
    pub avatar_url: Option<String>,
    pub bio: Option<String>,
    pub role: Option<String>,
    pub status: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<crate::domain::entities::users::Model> for UserProfile {
    fn from(user: crate::domain::entities::users::Model) -> Self {
        Self {
            id: user.id,
            email: user.email,
            username: user.username,
            nickname: user.nickname,
            avatar_url: user.avatar_url,
            bio: user.bio,
            role: user.role,
            status: user.status,
            created_at: user.created_at.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string(),
            updated_at: user.updated_at.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string(),
        }
    }
}
