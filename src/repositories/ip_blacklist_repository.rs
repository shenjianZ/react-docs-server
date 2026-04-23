use crate::domain::entities::ip_blacklist;
use anyhow::Result;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};

pub struct IpBlacklistRepository {
    db: DatabaseConnection,
}

impl IpBlacklistRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn is_blocked(&self, ip: &str) -> Result<bool> {
        let result = ip_blacklist::Entity::find()
            .filter(ip_blacklist::Column::IpAddress.eq(ip))
            .filter(ip_blacklist::Column::Enabled.eq(true))
            .one(&self.db)
            .await?;
        Ok(result.is_some())
    }
}
