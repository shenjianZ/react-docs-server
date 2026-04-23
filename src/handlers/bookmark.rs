use crate::domain::dto::bookmark::{BookmarkCheckQuery, CreateBookmarkRequest};
use crate::domain::vo::bookmark::BookmarkResult;
use crate::domain::vo::ApiResponse;
use crate::error::ErrorResponse;
use crate::repositories::bookmark_repository::BookmarkRepository;
use crate::services::bookmark_service::BookmarkService;
use crate::AppState;
use axum::{
    extract::{Extension, Path, Query, State},
    Json,
};
use serde_json::json;

pub async fn list_bookmarks(
    State(state): State<AppState>,
    Extension(user_id): Extension<String>,
) -> Result<Json<ApiResponse<Vec<BookmarkResult>>>, ErrorResponse> {
    let repo = BookmarkRepository::new(state.pool.clone());
    let rows = repo
        .list_by_user(&user_id)
        .await
        .map_err(|_| ErrorResponse::new("收藏列表获取失败，请稍后重试"))?;
    let data = rows.into_iter().map(BookmarkResult::from).collect();
    Ok(Json(ApiResponse::success_with_message(
        data,
        "收藏列表获取成功",
    )))
}

pub async fn create_bookmark(
    State(state): State<AppState>,
    Extension(user_id): Extension<String>,
    Json(payload): Json<CreateBookmarkRequest>,
) -> Result<Json<ApiResponse<BookmarkResult>>, ErrorResponse> {
    let service = BookmarkService::new(BookmarkRepository::new(state.pool.clone()));
    let model = service
        .create(payload, user_id)
        .await
        .map_err(|_| ErrorResponse::new("收藏失败，请稍后重试"))?;
    Ok(Json(ApiResponse::success_with_message(
        BookmarkResult::from(model),
        "收藏成功",
    )))
}

pub async fn check_bookmark(
    State(state): State<AppState>,
    Extension(user_id): Extension<String>,
    Query(query): Query<BookmarkCheckQuery>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ErrorResponse> {
    let repo = BookmarkRepository::new(state.pool.clone());
    let lang = query.lang.unwrap_or_else(|| "zh-cn".to_string());
    let bookmark = repo
        .find_by_page(&user_id, &query.page_slug, &lang)
        .await
        .map_err(|_| ErrorResponse::new("收藏状态获取失败，请稍后重试"))?;
    Ok(Json(ApiResponse::success_with_message(
        json!({
            "bookmarked": bookmark.is_some(),
            "id": bookmark.map(|item| item.id)
        }),
        "收藏状态获取成功",
    )))
}

pub async fn delete_bookmark(
    State(state): State<AppState>,
    Extension(user_id): Extension<String>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<()>>, ErrorResponse> {
    let repo = BookmarkRepository::new(state.pool.clone());
    repo.delete_by_id(&id, &user_id)
        .await
        .map_err(|_| ErrorResponse::new("取消收藏失败，请稍后重试"))?;
    Ok(Json(ApiResponse::success_with_message((), "已取消收藏")))
}
