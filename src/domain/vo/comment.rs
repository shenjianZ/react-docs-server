use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct CommentResult {
    pub id: String,
    pub page_slug: String,
    pub author_label: String,
    pub author_username: Option<String>,
    pub author_avatar_url: Option<String>,
    pub author_bio: Option<String>,
    pub can_edit: bool,
    pub can_reply: bool,
    pub parent_id: Option<String>,
    pub thread_root_id: String,
    pub reply_to_comment_id: Option<String>,
    pub reply_to_author_label: Option<String>,
    pub content: String,
    pub status: String,
    pub lang: String,
    pub like_count: i32,
    pub created_at: String,
}

impl From<crate::domain::entities::comments::Model> for CommentResult {
    fn from(model: crate::domain::entities::comments::Model) -> Self {
        Self::from_model(model, None)
    }
}

impl CommentResult {
    fn author_label(author: Option<&crate::domain::entities::users::Model>) -> String {
        author
            .and_then(|user| user.nickname.clone().or_else(|| user.username.clone()))
            .unwrap_or_else(|| "文档用户".to_string())
    }

    pub fn from_model(
        model: crate::domain::entities::comments::Model,
        current_user_id: Option<&str>,
    ) -> Self {
        let thread_root_id = model.id.clone();
        Self::from_model_with_author(
            model,
            current_user_id,
            None,
            thread_root_id,
            None,
            None,
        )
    }

    pub fn from_model_with_author(
        model: crate::domain::entities::comments::Model,
        current_user_id: Option<&str>,
        author: Option<&crate::domain::entities::users::Model>,
        thread_root_id: String,
        reply_to_comment_id: Option<String>,
        reply_to_author_label: Option<String>,
    ) -> Self {
        let can_edit = current_user_id.is_some_and(|user_id| user_id == model.user_id);
        let can_reply = current_user_id.is_some_and(|user_id| user_id != model.user_id);
        let author_label = Self::author_label(author);
        Self {
            id: model.id,
            page_slug: model.page_slug,
            author_label,
            author_username: author.and_then(|user| user.username.clone()),
            author_avatar_url: author.and_then(|user| user.avatar_url.clone()),
            author_bio: author.and_then(|user| user.bio.clone()),
            can_edit,
            can_reply,
            parent_id: model.parent_id,
            thread_root_id,
            reply_to_comment_id,
            reply_to_author_label,
            content: model.content,
            status: model.status,
            lang: model.lang,
            like_count: model.like_count,
            created_at: model
                .created_at
                .format("%Y-%m-%dT%H:%M:%S%.3fZ")
                .to_string(),
        }
    }
}
