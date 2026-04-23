use crate::domain::dto::analytics::{TrackDurationRequest, TrackPageViewRequest};
use crate::domain::vo::analytics::{AnalyticsOverview, PageViewResult, PopularPage};
use crate::domain::vo::ApiResponse;
use crate::error::ErrorResponse;
use crate::repositories::analytics_repository::AnalyticsRepository;
use crate::services::analytics_service::AnalyticsService;
use crate::AppState;
use axum::{extract::State, Json};
use std::collections::{HashMap, HashSet};

pub async fn track_view(
    State(state): State<AppState>,
    Json(payload): Json<TrackPageViewRequest>,
) -> Result<Json<ApiResponse<PageViewResult>>, ErrorResponse> {
    let service = AnalyticsService::new(AnalyticsRepository::new(state.pool.clone()));
    let model = service
        .track_view(payload)
        .await
        .map_err(|_| ErrorResponse::new("页面访问记录失败，请稍后重试"))?;
    Ok(Json(ApiResponse::success_with_message(
        PageViewResult {
            id: model.id,
            page_slug: model.page_slug,
            lang: model.lang,
            created_at: model
                .created_at
                .format("%Y-%m-%dT%H:%M:%S%.3fZ")
                .to_string(),
        },
        "页面访问已记录",
    )))
}

pub async fn track_duration(
    State(state): State<AppState>,
    Json(payload): Json<TrackDurationRequest>,
) -> Result<Json<ApiResponse<PageViewResult>>, ErrorResponse> {
    let service = AnalyticsService::new(AnalyticsRepository::new(state.pool.clone()));
    let model = service
        .track_duration(payload)
        .await
        .map_err(|_| ErrorResponse::new("页面停留时长记录失败，请稍后重试"))?;
    Ok(Json(ApiResponse::success_with_message(
        PageViewResult {
            id: model.id,
            page_slug: model.page_slug,
            lang: model.lang,
            created_at: model
                .created_at
                .format("%Y-%m-%dT%H:%M:%S%.3fZ")
                .to_string(),
        },
        "页面停留时长已记录",
    )))
}

pub async fn overview(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<AnalyticsOverview>>, ErrorResponse> {
    let repo = AnalyticsRepository::new(state.pool.clone());
    let views = repo
        .recent_views(10_000)
        .await
        .map_err(|_| ErrorResponse::new("访问统计概览获取失败，请稍后重试"))?;
    let unique_pages: HashSet<String> = views.iter().map(|view| view.page_slug.clone()).collect();
    Ok(Json(ApiResponse::success_with_message(
        AnalyticsOverview {
            total_views: repo
                .count()
                .await
                .map_err(|_| ErrorResponse::new("访问统计概览获取失败，请稍后重试"))?,
            unique_pages: unique_pages.len(),
        },
        "访问统计概览获取成功",
    )))
}

pub async fn popular_pages(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<PopularPage>>>, ErrorResponse> {
    let repo = AnalyticsRepository::new(state.pool.clone());
    let views = repo
        .recent_views(10_000)
        .await
        .map_err(|_| ErrorResponse::new("热门页面获取失败，请稍后重试"))?;
    let mut counts: HashMap<(String, String), (Option<String>, usize)> = HashMap::new();

    for view in views {
        let key = (view.page_slug, view.lang);
        let entry = counts.entry(key).or_insert((view.page_title, 0));
        entry.1 += 1;
    }

    let mut pages: Vec<PopularPage> = counts
        .into_iter()
        .map(|((page_slug, lang), (page_title, views))| PopularPage {
            page_slug,
            page_title,
            lang,
            views,
        })
        .collect();
    pages.sort_by(|a, b| b.views.cmp(&a.views));
    pages.truncate(20);
    Ok(Json(ApiResponse::success_with_message(
        pages,
        "热门页面获取成功",
    )))
}

pub async fn trends(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<PopularPage>>>, ErrorResponse> {
    popular_pages(State(state)).await
}
