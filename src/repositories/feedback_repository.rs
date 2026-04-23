use crate::domain::entities::page_feedbacks;
use anyhow::Result;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, DatabaseConnection, EntityTrait, QueryFilter,
    QueryOrder, Set,
};

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

    pub async fn find_by_subject(
        &self,
        user_id: Option<&str>,
        page_slug: &str,
        lang: &str,
        ip_address: Option<&str>,
    ) -> Result<Option<page_feedbacks::Model>> {
        let mut query = page_feedbacks::Entity::find()
            .filter(page_feedbacks::Column::PageSlug.eq(page_slug))
            .filter(page_feedbacks::Column::Lang.eq(lang))
            .order_by_desc(page_feedbacks::Column::CreatedAt);

        if let Some(user_id) = user_id {
            query = query.filter(page_feedbacks::Column::UserId.eq(user_id));
        } else if let Some(ip_address) = ip_address.filter(|value| !value.is_empty() && *value != "unknown") {
            query = query.filter(
                Condition::all()
                    .add(page_feedbacks::Column::UserId.is_null())
                    .add(page_feedbacks::Column::IpAddress.eq(ip_address)),
            );
        } else {
            return Ok(None);
        }

        query
            .one(&self.db)
            .await
            .map_err(|e| anyhow::anyhow!("查询反馈状态失败: {}", e))
    }

    pub fn active_model(
        id: String,
        page_slug: String,
        lang: String,
        value: String,
        comment: Option<String>,
        user_id: Option<String>,
        ip_address: Option<String>,
    ) -> page_feedbacks::ActiveModel {
        page_feedbacks::ActiveModel {
            id: Set(id),
            page_slug: Set(page_slug),
            lang: Set(lang),
            value: Set(value),
            comment: Set(comment),
            user_id: Set(user_id),
            ip_address: Set(ip_address),
            ..Default::default()
        }
    }
}
