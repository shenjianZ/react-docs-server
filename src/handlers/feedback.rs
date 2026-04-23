use crate::domain::dto::feedback::CreateFeedbackRequest;
use crate::domain::vo::feedback::FeedbackResult;
use crate::domain::vo::ApiResponse;
use crate::error::ErrorResponse;
use crate::infra::middleware::logging::{log_info, RequestId};
use crate::repositories::feedback_repository::FeedbackRepository;
use crate::services::feedback_service::FeedbackService;
use crate::AppState;
use axum::{
    extract::{Extension, State},
    Json,
};

pub async fn create_feedback(
    Extension(request_id): Extension<RequestId>,
    State(state): State<AppState>,
    Json(payload): Json<CreateFeedbackRequest>,
) -> Result<Json<ApiResponse<FeedbackResult>>, ErrorResponse> {
    log_info(&request_id, "提交页面反馈", &payload);

    let repo = FeedbackRepository::new(state.pool.clone());
    let service = FeedbackService::new(repo);

    match service.create(payload, None).await {
        Ok(model) => Ok(Json(ApiResponse::success_with_message(
            FeedbackResult::from(model),
            "反馈提交成功，感谢你的帮助",
        ))),
        Err(_) => Err(ErrorResponse::new("反馈提交失败，请稍后重试")),
    }
}
