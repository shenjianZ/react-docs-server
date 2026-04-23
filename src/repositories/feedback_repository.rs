use crate::domain::entities::page_feedbacks;
use anyhow::Result;
use sea_orm::{ActiveModelTrait, DatabaseConnection, Set};

pub struct FeedbackRepository {
    db: DatabaseConnection,
}

impl FeedbackRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn insert(
        &self,
        model: page_feedbacks::ActiveModel,
    ) -> Result<page_feedbacks::Model> {
        model
            .insert(&self.db)
            .await
            .map_err(|e| anyhow::anyhow!("插入反馈失败: {}", e))
    }

    pub fn active_model(
        id: String,
        page_slug: String,
        lang: String,
        value: String,
        comment: Option<String>,
        user_id: Option<String>,
    ) -> page_feedbacks::ActiveModel {
        page_feedbacks::ActiveModel {
            id: Set(id),
            page_slug: Set(page_slug),
            lang: Set(lang),
            value: Set(value),
            comment: Set(comment),
            user_id: Set(user_id),
            ..Default::default()
        }
    }
}
