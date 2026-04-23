use crate::domain::dto::comment::CreateCommentRequest;
use crate::domain::entities::comments;
use crate::repositories::comment_repository::CommentRepository;
use anyhow::Result;
use sea_orm::Set;
use uuid::Uuid;

pub struct CommentService {
    repo: CommentRepository,
}

impl CommentService {
    pub fn new(repo: CommentRepository) -> Self {
        Self { repo }
    }

    pub async fn create(
        &self,
        payload: CreateCommentRequest,
        user_id: String,
    ) -> Result<comments::Model> {
        let content = payload.content.trim().to_string();
        if content.is_empty() {
            return Err(anyhow::anyhow!("评论内容不能为空"));
        }

        let model = comments::ActiveModel {
            id: Set(Uuid::new_v4().to_string()),
            page_slug: Set(payload.page_slug),
            user_id: Set(user_id),
            parent_id: Set(payload.parent_id),
            content: Set(content),
            status: Set("active".to_string()),
            lang: Set(payload.lang.unwrap_or_else(|| "zh-cn".to_string())),
            like_count: Set(0),
            ..Default::default()
        };

        self.repo.insert(model).await
    }
}
