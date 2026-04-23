use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct PageViewResult {
    pub id: String,
    pub page_slug: String,
    pub lang: String,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct PopularPage {
    pub page_slug: String,
    pub page_title: Option<String>,
    pub lang: String,
    pub views: usize,
}

#[derive(Debug, Serialize)]
pub struct AnalyticsOverview {
    pub total_views: usize,
    pub unique_pages: usize,
}
