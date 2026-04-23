use crate::domain::entities::bookmarks;
use anyhow::Result;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder,
};

pub struct BookmarkRepository {
    db: DatabaseConnection,
}

impl BookmarkRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn list_by_user(&self, user_id: &str) -> Result<Vec<bookmarks::Model>> {
        bookmarks::Entity::find()
            .filter(bookmarks::Column::UserId.eq(user_id))
            .order_by_desc(bookmarks::Column::CreatedAt)
            .all(&self.db)
            .await
            .map_err(|e| anyhow::anyhow!("查询收藏失败: {}", e))
    }

    pub async fn insert(&self, model: bookmarks::ActiveModel) -> Result<bookmarks::Model> {
        model
            .insert(&self.db)
            .await
            .map_err(|e| anyhow::anyhow!("添加收藏失败: {}", e))
    }

    pub async fn find_by_page(
        &self,
        user_id: &str,
        page_slug: &str,
        lang: &str,
    ) -> Result<Option<bookmarks::Model>> {
        bookmarks::Entity::find()
            .filter(bookmarks::Column::UserId.eq(user_id))
            .filter(bookmarks::Column::PageSlug.eq(page_slug))
            .filter(bookmarks::Column::Lang.eq(lang))
            .one(&self.db)
            .await
            .map_err(|e| anyhow::anyhow!("检查收藏失败: {}", e))
    }

    pub async fn delete_by_id(&self, id: &str, user_id: &str) -> Result<()> {
        bookmarks::Entity::delete_many()
            .filter(bookmarks::Column::Id.eq(id))
            .filter(bookmarks::Column::UserId.eq(user_id))
            .exec(&self.db)
            .await
            .map_err(|e| anyhow::anyhow!("删除收藏失败: {}", e))?;

        Ok(())
    }
}
