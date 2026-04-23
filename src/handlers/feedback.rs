use crate::domain::dto::feedback::{CreateFeedbackRequest, FeedbackStatusQuery};
use crate::domain::vo::feedback::{FeedbackResult, FeedbackStatusResult};
use crate::domain::vo::ApiResponse;
use crate::error::ErrorResponse;
use crate::infra::middleware::logging::{log_info, RequestId};
use crate::repositories::feedback_repository::FeedbackRepository;
use crate::services::feedback_service::FeedbackService;
use crate::utils::jwt::TokenService;
use crate::AppState;
use axum::{
    extract::{Extension, Query, State},
    http::HeaderMap,
    Json,
};

fn client_ip(headers: &HeaderMap) -> Option<String> {
    headers
        .get("x-forwarded-for")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split(',').next())
        .or_else(|| headers.get("x-real-ip").and_then(|value| value.to_str().ok()))
        .or_else(|| headers.get("cf-connecting-ip").and_then(|value| value.to_str().ok()))
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty() && value != "unknown")
}

fn current_user_id(headers: &HeaderMap, state: &AppState) -> Option<String> {
    let token = headers
        .get("Authorization")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))?;

    TokenService::decode_user_id(token, &state.config.auth.jwt_secret).ok()
}

pub async fn feedback_status(
    headers: HeaderMap,
    State(state): State<AppState>,
    Query(query): Query<FeedbackStatusQuery>,
) -> Result<Json<ApiResponse<FeedbackStatusResult>>, ErrorResponse> {
    let repo = FeedbackRepository::new(state.pool.clone());
    let service = FeedbackService::new(repo);
    let model = service
        .status(
            query.slug.unwrap_or_else(|| "index".to_string()),
            query.lang.unwrap_or_else(|| "zh-cn".to_string()),
            current_user_id(&headers, &state),
            client_ip(&headers),
        )
        .await
        .map_err(|_| ErrorResponse::new("反馈状态获取失败，请稍后重试"))?;

    Ok(Json(ApiResponse::success_with_message(
        FeedbackStatusResult::from(model),
        "反馈状态获取成功",
    )))
}

pub async fn create_feedback(
    Extension(request_id): Extension<RequestId>,
    headers: HeaderMap,
    State(state): State<AppState>,
    Json(payload): Json<CreateFeedbackRequest>,
) -> Result<Json<ApiResponse<FeedbackResult>>, ErrorResponse> {
    log_info(&request_id, "提交页面反馈", &payload);

    let repo = FeedbackRepository::new(state.pool.clone());
    let service = FeedbackService::new(repo);

    match service
        .create(
            payload,
            current_user_id(&headers, &state),
            client_ip(&headers),
        )
        .await
    {
        Ok(model) => Ok(Json(ApiResponse::success_with_message(
            FeedbackResult::from(model),
            "反馈提交成功，感谢你的帮助",
        ))),
        Err(_) => Err(ErrorResponse::new("反馈提交失败，请稍后重试")),
    }
}
