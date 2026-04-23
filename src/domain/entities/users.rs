use sea_orm::entity::prelude::*;
use sea_orm::Set;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    #[sea_orm(unique)]
    pub email: String,
    #[sea_orm(unique, nullable)]
    pub username: Option<String>,
    pub nickname: Option<String>,
    pub avatar_url: Option<String>,
    pub bio: Option<String>,
    pub role: Option<String>,
    pub status: Option<String>,
    pub last_login_at: Option<DateTime>,
    pub password_hash: String,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

#[async_trait::async_trait]
impl ActiveModelBehavior for ActiveModel {
    /// 在保存前自动填充时间戳
    async fn before_save<C>(self, _db: &C, insert: bool) -> Result<Self, DbErr>
    where
        C: ConnectionTrait,
    {
        let mut this = self;
        let now = chrono::Utc::now().naive_utc();

        if insert {
            // 插入时：设置创建时间和更新时间
            this.created_at = Set(now);
            this.updated_at = Set(now);
        } else {
            // 更新时：只更新更新时间
            this.updated_at = Set(now);
        }

        Ok(this)
    }
}
