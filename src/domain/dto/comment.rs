use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct CommentListQuery {
    pub page_slug: String,
    pub lang: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateCommentRequest {
    pub page_slug: String,
    pub parent_id: Option<String>,
    pub content: String,
    pub lang: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCommentRequest {
    pub content: String,
}
