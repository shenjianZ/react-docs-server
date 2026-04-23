use crate::domain::entities::users;
use anyhow::Result;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter,
    Set,
};

pub struct UserRepository {
    db: DatabaseConnection,
}

impl UserRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn find_by_email(&self, email: &str) -> Result<Option<users::Model>> {
        let user = users::Entity::find()
            .filter(users::Column::Email.eq(email))
            .one(&self.db)
            .await
            .map_err(|e| anyhow::anyhow!("查询失败: {}", e))?;

        Ok(user)
    }

    pub async fn find_by_id(&self, id: &str) -> Result<Option<users::Model>> {
        let user = users::Entity::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(|e| anyhow::anyhow!("查询失败: {}", e))?;

        Ok(user)
    }

    pub async fn count_by_email(&self, email: &str) -> Result<i64> {
        let count = users::Entity::find()
            .filter(users::Column::Email.eq(email))
            .count(&self.db)
            .await
            .map_err(|e| anyhow::anyhow!("查询失败: {}", e))?;

        Ok(count as i64)
    }

    pub async fn count_by_id(&self, id: &str) -> Result<i64> {
        let count = users::Entity::find()
            .filter(users::Column::Id.eq(id))
            .count(&self.db)
            .await
            .map_err(|e| anyhow::anyhow!("查询失败: {}", e))?;

        Ok(count as i64)
    }

    pub async fn get_password_hash(&self, email: &str) -> Result<Option<String>> {
        let user = users::Entity::find()
            .filter(users::Column::Email.eq(email))
            .one(&self.db)
            .await
            .map_err(|e| anyhow::anyhow!("查询失败: {}", e))?;

        Ok(user.map(|u| u.password_hash))
    }

    pub async fn get_password_hash_by_user_id(&self, user_id: &str) -> Result<Option<String>> {
        let user = users::Entity::find_by_id(user_id)
            .one(&self.db)
            .await
            .map_err(|e| anyhow::anyhow!("查询失败: {}", e))?;

        Ok(user.map(|u| u.password_hash))
    }

    pub async fn insert_local(
        &self,
        id: String,
        email: String,
        password_hash: String,
        email_verified: bool,
    ) -> Result<users::Model> {
        let username = format!("user_{}", id);
        let nickname = email.split('@').next().unwrap_or("文档用户").to_string();
        let user_model = users::ActiveModel {
            id: Set(id),
            email: Set(email),
            email_verified: Set(email_verified),
            password_set: Set(true),
            username: Set(Some(username)),
            nickname: Set(Some(nickname)),
            avatar_url: Set(Some("/api/avatar/default-avatar.jpg".to_string())),
            bio: Set(Some("这个用户还没有填写个人简介。".to_string())),
            role: Set(Some("user".to_string())),
            status: Set(Some("active".to_string())),
            password_hash: Set(password_hash),
            ..Default::default()
        };

        user_model
            .insert(&self.db)
            .await
            .map_err(|e| anyhow::anyhow!("插入失败: {}", e))
    }

    pub async fn insert_oauth_user(
        &self,
        id: String,
        email: String,
        password_hash: String,
        email_verified: bool,
        nickname: Option<String>,
        avatar_url: Option<String>,
        bio: Option<String>,
    ) -> Result<users::Model> {
        let default_nickname = email.split('@').next().unwrap_or("文档用户").to_string();
        let user_model = users::ActiveModel {
            id: Set(id.clone()),
            email: Set(email),
            email_verified: Set(email_verified),
            password_set: Set(false),
            username: Set(Some(format!("user_{}", id))),
            nickname: Set(Some(nickname.unwrap_or(default_nickname))),
            avatar_url: Set(avatar_url.or(Some("/api/avatar/default-avatar.jpg".to_string()))),
            bio: Set(bio.or(Some("这个用户还没有填写个人简介。".to_string()))),
            role: Set(Some("user".to_string())),
            status: Set(Some("active".to_string())),
            password_hash: Set(password_hash),
            ..Default::default()
        };

        user_model
            .insert(&self.db)
            .await
            .map_err(|e| anyhow::anyhow!("插入失败: {}", e))
    }

    pub async fn update_last_login(&self, id: &str) -> Result<users::Model> {
        let user = users::Entity::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("用户不存在"))?;
        let mut active: users::ActiveModel = user.into();
        active.last_login_at = Set(Some(chrono::Utc::now().naive_utc()));
        active
            .update(&self.db)
            .await
            .map_err(|e| anyhow::anyhow!("更新失败: {}", e))
    }

    pub async fn update_email_verified(&self, id: &str, email_verified: bool) -> Result<users::Model> {
        let user = users::Entity::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("用户不存在"))?;
        let mut active: users::ActiveModel = user.into();
        active.email_verified = Set(email_verified);
        active
            .update(&self.db)
            .await
            .map_err(|e| anyhow::anyhow!("更新失败: {}", e))
    }

    pub async fn delete_by_id(&self, id: &str) -> Result<()> {
        users::Entity::delete_by_id(id)
            .exec(&self.db)
            .await
            .map_err(|e| anyhow::anyhow!("删除失败: {}", e))?;

        Ok(())
    }

    pub async fn update_profile(
        &self,
        id: &str,
        username: Option<String>,
        nickname: Option<String>,
        bio: Option<String>,
    ) -> Result<users::Model> {
        let user = users::Entity::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("用户不存在"))?;
        let mut active: users::ActiveModel = user.into();
        active.username = Set(username);
        active.nickname = Set(nickname);
        active.bio = Set(bio);
        active.update(&self.db).await.map_err(|e| anyhow::anyhow!("更新失败: {}", e))
    }

    pub async fn update_avatar(&self, id: &str, avatar_url: String) -> Result<users::Model> {
        let user = users::Entity::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("用户不存在"))?;
        let mut active: users::ActiveModel = user.into();
        active.avatar_url = Set(Some(avatar_url));
        active.update(&self.db).await.map_err(|e| anyhow::anyhow!("更新失败: {}", e))
    }
}
