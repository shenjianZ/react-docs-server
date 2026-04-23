use crate::domain::dto::analytics::{TrackDurationRequest, TrackPageViewRequest};
use crate::domain::entities::page_views;
use crate::repositories::analytics_repository::AnalyticsRepository;
use anyhow::Result;
use sea_orm::Set;
use uuid::Uuid;

pub struct AnalyticsService {
    repo: AnalyticsRepository,
}

impl AnalyticsService {
    pub fn new(repo: AnalyticsRepository) -> Self {
        Self { repo }
    }

    pub async fn track_view(&self, payload: TrackPageViewRequest) -> Result<page_views::Model> {
        self.insert(
            payload.page_slug,
            payload.page_title,
            payload.lang,
            payload.path,
            payload.referrer,
            None,
        )
        .await
    }

    pub async fn track_duration(&self, payload: TrackDurationRequest) -> Result<page_views::Model> {
        let duration = payload.duration_seconds.max(0);
        self.insert(
            payload.page_slug,
            payload.page_title,
            payload.lang,
            payload.path,
            None,
            Some(duration),
        )
        .await
    }

    async fn insert(
        &self,
        page_slug: String,
        page_title: Option<String>,
        lang: Option<String>,
        path: Option<String>,
        referrer: Option<String>,
        duration_seconds: Option<i32>,
    ) -> Result<page_views::Model> {
        let model = page_views::ActiveModel {
            id: Set(Uuid::new_v4().to_string()),
            user_id: Set(None),
            page_slug: Set(page_slug),
            page_title: Set(page_title),
            lang: Set(lang.unwrap_or_else(|| "zh-cn".to_string())),
            path: Set(path),
            referrer: Set(referrer),
            duration_seconds: Set(duration_seconds),
            ..Default::default()
        };

        self.repo.insert(model).await
    }
}
