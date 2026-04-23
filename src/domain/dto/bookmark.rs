use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct BookmarkCheckQuery {
    pub page_slug: String,
    pub lang: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateBookmarkRequest {
    pub page_slug: String,
    pub page_title: Option<String>,
    pub folder: Option<String>,
    pub notes: Option<String>,
    pub lang: Option<String>,
}
