use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct BookmarkResult {
    pub id: String,
    pub page_slug: String,
    pub page_title: Option<String>,
    pub folder: Option<String>,
    pub notes: Option<String>,
    pub lang: String,
    pub created_at: String,
}

impl From<crate::domain::entities::bookmarks::Model> for BookmarkResult {
    fn from(model: crate::domain::entities::bookmarks::Model) -> Self {
        Self {
            id: model.id,
            page_slug: model.page_slug,
            page_title: model.page_title,
            folder: model.folder,
            notes: model.notes,
            lang: model.lang,
            created_at: model
                .created_at
                .format("%Y-%m-%dT%H:%M:%S%.3fZ")
                .to_string(),
        }
    }
}
