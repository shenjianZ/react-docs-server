use crate::domain::entities::audit_logs;
use anyhow::Result;
use sea_orm::{ActiveModelTrait, DatabaseConnection, Set};
use uuid::Uuid;

pub struct AuditLogRepository {
    db: DatabaseConnection,
}

impl AuditLogRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn insert(
        &self,
        user_id: Option<String>,
        action: &str,
        target: Option<String>,
        ip_address: Option<String>,
        detail: Option<String>,
    ) -> Result<()> {
        let model = audit_logs::ActiveModel {
            id: Set(Uuid::new_v4().to_string()),
            user_id: Set(user_id),
            action: Set(action.to_string()),
            target: Set(target),
            ip_address: Set(ip_address),
            detail: Set(detail),
            ..Default::default()
        };
        model.insert(&self.db).await?;
        Ok(())
    }
}
