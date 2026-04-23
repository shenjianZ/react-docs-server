use crate::domain::dto::comment::{CommentListQuery, CreateCommentRequest, UpdateCommentRequest};
use crate::domain::entities::{comments, users};
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
use std::collections::{HashMap, HashSet};

struct CommentThreadContext {
    thread_root_id: String,
    reply_to_comment_id: Option<String>,
    reply_to_author_label: Option<String>,
}

fn optional_user_id(headers: &HeaderMap, state: &AppState) -> Option<String> {
    let token = headers
        .get("Authorization")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))?;

    TokenService::decode_user_id(token, &state.config.auth.jwt_secret).ok()
}

fn author_label(author: Option<&users::Model>) -> String {
    author
        .and_then(|user| user.nickname.clone().or_else(|| user.username.clone()))
        .unwrap_or_else(|| "文档用户".to_string())
}

fn resolve_thread_context(
    comment: &comments::Model,
    comments_by_id: &HashMap<String, comments::Model>,
    authors_by_user_id: &HashMap<String, Option<users::Model>>,
) -> CommentThreadContext {
    let reply_to_comment_id = comment.parent_id.clone();
    let reply_to_author_label = comment.parent_id.as_ref().and_then(|parent_id| {
        comments_by_id.get(parent_id).map(|parent| {
            let parent_author = authors_by_user_id
                .get(&parent.user_id)
                .and_then(|author| author.as_ref());
            author_label(parent_author)
        })
    });

    let mut thread_root_id = comment.id.clone();
    let mut current_id = comment.id.clone();
    let mut visited = HashSet::from([current_id.clone()]);

    loop {
        let Some(current) = comments_by_id.get(&current_id) else {
            break;
        };
        let Some(parent_id) = current.parent_id.as_ref() else {
            break;
        };
        if !visited.insert(parent_id.clone()) {
            break;
        }
        let Some(parent) = comments_by_id.get(parent_id) else {
            break;
        };
        thread_root_id = parent.id.clone();
        current_id = parent.id.clone();
    }

    CommentThreadContext {
        thread_root_id,
        reply_to_comment_id,
        reply_to_author_label,
    }
}

async fn load_author_cache(
    user_repo: &UserRepository,
    cache: &mut HashMap<String, Option<users::Model>>,
    user_id: &str,
) {
    if cache.contains_key(user_id) {
        return;
    }
    let author = user_repo.find_by_id(user_id).await.ok().flatten();
    cache.insert(user_id.to_string(), author);
}

async fn resolve_thread_context_for_comment(
    repo: &CommentRepository,
    user_repo: &UserRepository,
    comment: &comments::Model,
) -> Result<CommentThreadContext, ErrorResponse> {
    let mut comments_by_id = HashMap::from([(comment.id.clone(), comment.clone())]);
    let mut current = comment.clone();
    let mut visited = HashSet::from([comment.id.clone()]);

    while let Some(parent_id) = current.parent_id.clone() {
        if !visited.insert(parent_id.clone()) {
            break;
        }
        let Some(parent) = repo
            .find_by_id(&parent_id)
            .await
            .map_err(|_| ErrorResponse::new("评论线程解析失败，请稍后重试"))?
        else {
            break;
        };
        comments_by_id.insert(parent.id.clone(), parent.clone());
        current = parent;
    }

    let mut authors_by_user_id = HashMap::new();
    for item in comments_by_id.values() {
        load_author_cache(user_repo, &mut authors_by_user_id, &item.user_id).await;
    }

    Ok(resolve_thread_context(
        comment,
        &comments_by_id,
        &authors_by_user_id,
    ))
}

fn collect_comment_subtree_ids(root_id: &str, comments: &[comments::Model]) -> Vec<String> {
    let mut children_map: HashMap<String, Vec<String>> = HashMap::new();
    for comment in comments {
        if let Some(parent_id) = comment.parent_id.as_ref() {
            children_map
                .entry(parent_id.clone())
                .or_default()
                .push(comment.id.clone());
        }
    }

    let mut ids = Vec::new();
    let mut stack = vec![root_id.to_string()];
    let mut visited = HashSet::new();

    while let Some(comment_id) = stack.pop() {
        if !visited.insert(comment_id.clone()) {
            continue;
        }
        ids.push(comment_id.clone());
        if let Some(children) = children_map.get(&comment_id) {
            stack.extend(children.iter().cloned());
        }
    }

    ids
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

    let comments_by_id: HashMap<String, comments::Model> = comments
        .iter()
        .cloned()
        .map(|comment| (comment.id.clone(), comment))
        .collect();
    let mut authors_by_user_id = HashMap::new();
    for comment in &comments {
        load_author_cache(&user_repo, &mut authors_by_user_id, &comment.user_id).await;
    }

    let mut data = Vec::with_capacity(comments.len());
    for comment in comments {
        let author = authors_by_user_id
            .get(&comment.user_id)
            .and_then(|author| author.as_ref());
        let thread = resolve_thread_context(&comment, &comments_by_id, &authors_by_user_id);
        data.push(CommentResult::from_model_with_author(
            comment,
            current_user_id.as_deref(),
            author,
            thread.thread_root_id,
            thread.reply_to_comment_id,
            thread.reply_to_author_label,
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
    let user_repo = UserRepository::new(state.pool.clone());
    let resolved_lang = payload.lang.clone().unwrap_or_else(|| "zh-cn".to_string());

    if let Some(parent_id) = payload.parent_id.as_deref() {
        let parent = repo
            .find_by_id(parent_id)
            .await
            .map_err(|_| ErrorResponse::new("回复目标获取失败，请稍后重试"))?
            .ok_or_else(|| ErrorResponse::not_found("回复的评论不存在"))?;

        if parent.user_id == user_id {
            return Err(ErrorResponse::forbidden("不能直接回复自己的评论"));
        }
        if parent.page_slug != payload.page_slug || parent.lang != resolved_lang {
            return Err(ErrorResponse::new("回复目标与当前页面不匹配"));
        }
    }

    let service = CommentService::new(CommentRepository::new(state.pool.clone()));
    let model = service
        .create(
            CreateCommentRequest {
                page_slug: payload.page_slug,
                parent_id: payload.parent_id,
                content: payload.content,
                lang: Some(resolved_lang),
            },
            user_id.clone(),
        )
        .await
        .map_err(|e| ErrorResponse::new(e.to_string()))?;
    let author = user_repo.find_by_id(&user_id).await.ok().flatten();
    let thread = resolve_thread_context_for_comment(&repo, &user_repo, &model).await?;
    Ok(Json(ApiResponse::success_with_message(
        CommentResult::from_model_with_author(
            model,
            Some(&user_id),
            author.as_ref(),
            thread.thread_root_id,
            thread.reply_to_comment_id,
            thread.reply_to_author_label,
        ),
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
    let thread = resolve_thread_context_for_comment(&repo, &user_repo, &model).await?;
    Ok(Json(ApiResponse::success_with_message(
        CommentResult::from_model_with_author(
            model,
            Some(&user_id),
            author.as_ref(),
            thread.thread_root_id,
            thread.reply_to_comment_id,
            thread.reply_to_author_label,
        ),
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

    let comments_in_scope = repo
        .list(&comment.page_slug, &comment.lang)
        .await
        .map_err(|_| ErrorResponse::new("评论删除失败，请稍后重试"))?;
    let mut subtree_ids = collect_comment_subtree_ids(&id, &comments_in_scope);
    subtree_ids.reverse();

    for comment_id in subtree_ids {
        repo.delete_by_id(&comment_id)
            .await
            .map_err(|_| ErrorResponse::new("评论删除失败，请稍后重试"))?;
    }

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
    let comment = repo
        .find_by_id(&id)
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
    let thread = resolve_thread_context_for_comment(&repo, &user_repo, &model).await?;
    Ok(Json(ApiResponse::success_with_message(
        CommentResult::from_model_with_author(
            model,
            Some(&user_id),
            author.as_ref(),
            thread.thread_root_id,
            thread.reply_to_comment_id,
            thread.reply_to_author_label,
        ),
        "点赞成功",
    )))
}
