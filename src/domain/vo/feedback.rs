use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct FeedbackResult {
    pub id: String,
    pub page_slug: String,
    pub lang: String,
    pub value: String,
    pub created_at: String,
}

impl From<crate::domain::entities::page_feedbacks::Model> for FeedbackResult {
    fn from(model: crate::domain::entities::page_feedbacks::Model) -> Self {
        Self {
            id: model.id,
            page_slug: model.page_slug,
            lang: model.lang,
            value: model.value,
            created_at: model
                .created_at
                .format("%Y-%m-%dT%H:%M:%S%.3fZ")
                .to_string(),
        }
    }
}
