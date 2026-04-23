use crate::domain::entities::comments;
use anyhow::Result;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, Set,
};

pub struct CommentRepository {
    db: DatabaseConnection,
}

impl CommentRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn list(&self, page_slug: &str, lang: &str) -> Result<Vec<comments::Model>> {
        comments::Entity::find()
            .filter(comments::Column::PageSlug.eq(page_slug))
            .filter(comments::Column::Lang.eq(lang))
            .filter(comments::Column::Status.eq("active"))
            .order_by_asc(comments::Column::CreatedAt)
            .all(&self.db)
            .await
            .map_err(|e| anyhow::anyhow!("查询评论失败: {}", e))
    }

    pub async fn find_by_id(&self, id: &str) -> Result<Option<comments::Model>> {
        comments::Entity::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(|e| anyhow::anyhow!("查询评论失败: {}", e))
    }

    pub async fn insert(&self, model: comments::ActiveModel) -> Result<comments::Model> {
        model
            .insert(&self.db)
            .await
            .map_err(|e| anyhow::anyhow!("插入评论失败: {}", e))
    }

    pub async fn update_content(&self, id: &str, content: String) -> Result<comments::Model> {
        let model = comments::Entity::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("评论不存在"))?;

        let mut active: comments::ActiveModel = model.into();
        active.content = Set(content);
        active
            .update(&self.db)
            .await
            .map_err(|e| anyhow::anyhow!("更新评论失败: {}", e))
    }

    pub async fn delete_by_id(&self, id: &str) -> Result<()> {
        comments::Entity::delete_by_id(id)
            .exec(&self.db)
            .await
            .map_err(|e| anyhow::anyhow!("删除评论失败: {}", e))?;

        Ok(())
    }

    pub async fn increment_like(&self, id: &str) -> Result<comments::Model> {
        let model = comments::Entity::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("评论不存在"))?;

        let mut active: comments::ActiveModel = model.clone().into();
        active.like_count = Set(model.like_count + 1);
        active
            .update(&self.db)
            .await
            .map_err(|e| anyhow::anyhow!("点赞失败: {}", e))
    }
}
