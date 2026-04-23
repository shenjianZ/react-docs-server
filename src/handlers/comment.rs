use crate::domain::dto::comment::{CommentListQuery, CreateCommentRequest, UpdateCommentRequest};
use crate::domain::vo::comment::CommentResult;
use crate::domain::vo::ApiResponse;
use crate::error::ErrorResponse;
use crate::infra::redis::redis_key::{BusinessType, RedisKey};
use crate::repositories::comment_repository::CommentRepository;
use crate::repositories::user_repository::UserRepository;
use crate::services::comment_service::CommentService;
use crate::utils::jwt::TokenService;
use crate::AppState;
use axum::{
    extract::{Extension, Path, Query, State},
    http::HeaderMap,
    Json,
};

fn optional_user_id(headers: &HeaderMap, state: &AppState) -> Option<String> {
    let token = headers
        .get("Authorization")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))?;

    TokenService::decode_user_id(token, &state.config.auth.jwt_secret).ok()
}

pub async fn list_comments(
    headers: HeaderMap,
    State(state): State<AppState>,
    Query(query): Query<CommentListQuery>,
) -> Result<Json<ApiResponse<Vec<CommentResult>>>, ErrorResponse> {
    let repo = CommentRepository::new(state.pool.clone());
    let lang = query.lang.unwrap_or_else(|| "zh-cn".to_string());
    let comments = repo
        .list(&query.page_slug, &lang)
        .await
        .map_err(|_| ErrorResponse::new("评论列表获取失败，请稍后重试"))?;
    let current_user_id = optional_user_id(&headers, &state);
    let user_repo = UserRepository::new(state.pool.clone());
    let mut data = Vec::with_capacity(comments.len());
    for comment in comments {
        let author = user_repo.find_by_id(&comment.user_id).await.ok().flatten();
        data.push(CommentResult::from_model_with_author(
            comment,
            current_user_id.as_deref(),
            author.as_ref(),
        ));
    }
    Ok(Json(ApiResponse::success_with_message(
        data,
        "评论列表获取成功",
    )))
}

pub async fn create_comment(
    State(state): State<AppState>,
    Extension(user_id): Extension<String>,
    Json(payload): Json<CreateCommentRequest>,
) -> Result<Json<ApiResponse<CommentResult>>, ErrorResponse> {
    let repo = CommentRepository::new(state.pool.clone());
    let service = CommentService::new(repo);
    let model = service
        .create(payload, user_id.clone())
        .await
        .map_err(|_| ErrorResponse::new("评论发表失败，请稍后重试"))?;
    let user_repo = UserRepository::new(state.pool.clone());
    let author = user_repo.find_by_id(&user_id).await.ok().flatten();
    Ok(Json(ApiResponse::success_with_message(
        CommentResult::from_model_with_author(model, Some(&user_id), author.as_ref()),
        "评论发表成功",
    )))
}

pub async fn update_comment(
    State(state): State<AppState>,
    Extension(user_id): Extension<String>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateCommentRequest>,
) -> Result<Json<ApiResponse<CommentResult>>, ErrorResponse> {
    let repo = CommentRepository::new(state.pool.clone());
    let comment = repo
        .find_by_id(&id)
        .await
        .map_err(|_| ErrorResponse::new("评论获取失败，请稍后重试"))?
        .ok_or_else(|| ErrorResponse::not_found("评论不存在"))?;
    if comment.user_id != user_id {
        return Err(ErrorResponse::forbidden("无权修改此评论"));
    }
    let model = repo
        .update_content(&id, payload.content)
        .await
        .map_err(|_| ErrorResponse::new("评论更新失败，请稍后重试"))?;
    let user_repo = UserRepository::new(state.pool.clone());
    let author = user_repo.find_by_id(&user_id).await.ok().flatten();
    Ok(Json(ApiResponse::success_with_message(
        CommentResult::from_model_with_author(model, Some(&user_id), author.as_ref()),
        "评论更新成功",
    )))
}

pub async fn delete_comment(
    State(state): State<AppState>,
    Extension(user_id): Extension<String>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<()>>, ErrorResponse> {
    let repo = CommentRepository::new(state.pool.clone());
    let comment = repo
        .find_by_id(&id)
        .await
        .map_err(|_| ErrorResponse::new("评论获取失败，请稍后重试"))?
        .ok_or_else(|| ErrorResponse::not_found("评论不存在"))?;
    if comment.user_id != user_id {
        return Err(ErrorResponse::forbidden("无权删除此评论"));
    }
    repo.delete_by_id(&id)
        .await
        .map_err(|_| ErrorResponse::new("评论删除失败，请稍后重试"))?;
    Ok(Json(ApiResponse::success_with_message((), "评论删除成功")))
}

pub async fn like_comment(
    State(state): State<AppState>,
    Extension(user_id): Extension<String>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<CommentResult>>, ErrorResponse> {
    let like_key = RedisKey::new(BusinessType::Comment)
        .add_identifier("like")
        .add_identifier(&id)
        .add_identifier(&user_id);
    if state
        .redis_client
        .exists_key(&like_key)
        .await
        .unwrap_or(false)
    {
        return Err(ErrorResponse::new("已点赞过此评论"));
    }
    let repo = CommentRepository::new(state.pool.clone());
    let comment = repo.find_by_id(&id)
        .await
        .map_err(|_| ErrorResponse::new("评论获取失败，请稍后重试"))?
        .ok_or_else(|| ErrorResponse::not_found("评论不存在"))?;
    let model = repo
        .increment_like(&id)
        .await
        .map_err(|_| ErrorResponse::new("点赞失败，请稍后重试"))?;
    let _ = state.redis_client.set_key_ex(&like_key, "1", 86400).await;
    let user_repo = UserRepository::new(state.pool.clone());
    let author = user_repo.find_by_id(&comment.user_id).await.ok().flatten();
    Ok(Json(ApiResponse::success_with_message(
        CommentResult::from_model_with_author(model, Some(&user_id), author.as_ref()),
        "点赞成功",
    )))
}
