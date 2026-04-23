use crate::domain::dto::bookmark::CreateBookmarkRequest;
use crate::domain::entities::bookmarks;
use crate::repositories::bookmark_repository::BookmarkRepository;
use anyhow::Result;
use sea_orm::Set;
use uuid::Uuid;

pub struct BookmarkService {
    repo: BookmarkRepository,
}

impl BookmarkService {
    pub fn new(repo: BookmarkRepository) -> Self {
        Self { repo }
    }

    pub async fn create(
        &self,
        payload: CreateBookmarkRequest,
        user_id: String,
    ) -> Result<bookmarks::Model> {
        let model = bookmarks::ActiveModel {
            id: Set(Uuid::new_v4().to_string()),
            user_id: Set(user_id),
            page_slug: Set(payload.page_slug),
            page_title: Set(payload.page_title),
            folder: Set(payload.folder),
            notes: Set(payload.notes),
            lang: Set(payload.lang.unwrap_or_else(|| "zh-cn".to_string())),
            ..Default::default()
        };

        self.repo.insert(model).await
    }
}
