use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateFeedbackRequest {
    pub value: String,
    pub comment: Option<String>,
    pub lang: Option<String>,
    pub slug: Option<String>,
}
