use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrackPageViewRequest {
    pub page_slug: String,
    pub page_title: Option<String>,
    pub lang: Option<String>,
    pub path: Option<String>,
    pub referrer: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrackDurationRequest {
    pub page_slug: String,
    pub page_title: Option<String>,
    pub lang: Option<String>,
    pub path: Option<String>,
    pub duration_seconds: i32,
}
