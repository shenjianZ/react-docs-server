use crate::domain::entities::page_views;
use anyhow::Result;
use sea_orm::{
    ActiveModelTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryOrder, QuerySelect,
};

pub struct AnalyticsRepository {
    db: DatabaseConnection,
}

impl AnalyticsRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn insert(&self, model: page_views::ActiveModel) -> Result<page_views::Model> {
        model
            .insert(&self.db)
            .await
            .map_err(|e| anyhow::anyhow!("记录访问失败: {}", e))
    }

    pub async fn recent_views(&self, limit: u64) -> Result<Vec<page_views::Model>> {
        page_views::Entity::find()
            .order_by_desc(page_views::Column::CreatedAt)
            .limit(limit)
            .all(&self.db)
            .await
            .map_err(|e| anyhow::anyhow!("查询访问统计失败: {}", e))
    }

    pub async fn count(&self) -> Result<usize> {
        page_views::Entity::find()
            .count(&self.db)
            .await
            .map(|count| count as usize)
            .map_err(|e| anyhow::anyhow!("统计访问量失败: {}", e))
    }
}
