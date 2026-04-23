use crate::domain::entities::oauth_accounts;
use anyhow::Result;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};

pub struct OAuthAccountRepository {
    db: DatabaseConnection,
}

impl OAuthAccountRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    fn account_id(provider: &str, provider_user_id: &str) -> String {
        format!("{}:{}", provider, provider_user_id)
    }

    pub async fn find_by_provider_user_id(
        &self,
        provider: &str,
        provider_user_id: &str,
    ) -> Result<Option<oauth_accounts::Model>> {
        oauth_accounts::Entity::find_by_id(Self::account_id(provider, provider_user_id))
            .one(&self.db)
            .await
            .map_err(|e| anyhow::anyhow!("查询 OAuth 账号失败: {}", e))
    }

    pub async fn find_by_user_id_and_provider(
        &self,
        user_id: &str,
        provider: &str,
    ) -> Result<Option<oauth_accounts::Model>> {
        oauth_accounts::Entity::find()
            .filter(oauth_accounts::Column::UserId.eq(user_id))
            .filter(oauth_accounts::Column::Provider.eq(provider))
            .one(&self.db)
            .await
            .map_err(|e| anyhow::anyhow!("查询 OAuth 账号失败: {}", e))
    }

    pub async fn insert(
        &self,
        user_id: String,
        provider: String,
        provider_user_id: String,
        provider_email: Option<String>,
    ) -> Result<oauth_accounts::Model> {
        oauth_accounts::ActiveModel {
            id: Set(Self::account_id(&provider, &provider_user_id)),
            user_id: Set(user_id),
            provider: Set(provider),
            provider_user_id: Set(provider_user_id),
            provider_email: Set(provider_email),
            ..Default::default()
        }
        .insert(&self.db)
        .await
        .map_err(|e| anyhow::anyhow!("创建 OAuth 账号失败: {}", e))
    }
}
