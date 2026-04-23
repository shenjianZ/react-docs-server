use crate::domain::dto::auth::{
    DeleteUserRequest, LoginRequest, RefreshRequest, RegisterRequest, UpdateProfileRequest,
};
use crate::domain::vo::auth::{LoginResult, RefreshResult, RegisterResult};
use crate::domain::vo::user::UserProfile;
use crate::domain::vo::ApiResponse;
use crate::error::ErrorResponse;
use crate::infra::middleware::logging::{log_info, RequestId};
use crate::repositories::audit_log_repository::AuditLogRepository;
use crate::repositories::user_repository::UserRepository;
use crate::services::auth_service::AuthService;
use crate::AppState;
use axum::{
    extract::{Extension, Multipart, State},
    http::HeaderMap,
    Json,
};
use serde_json::json;
use uuid::Uuid;

fn client_ip(headers: &HeaderMap) -> String {
    headers
        .get("x-forwarded-for")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split(',').next())
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "unknown".to_string())
}

fn avatar_extension(content_type: Option<&str>) -> Option<&'static str> {
    match content_type {
        Some("image/jpeg") => Some("jpg"),
        Some("image/png") => Some("png"),
        Some("image/webp") => Some("webp"),
        _ => None,
    }
}

async fn check_rate_limit(
    state: &AppState,
    key: &str,
    limit: i64,
    seconds: u64,
) -> Result<(), ErrorResponse> {
    let count = state
        .redis_client
        .incr(key)
        .await
        .map_err(|_| ErrorResponse::new("请求频率校验失败，请稍后重试"))?;
    if count == 1 {
        state
            .redis_client
            .expire(key, seconds)
            .await
            .map_err(|_| ErrorResponse::new("请求频率校验失败，请稍后重试"))?;
    }
    if count > limit {
        return Err(ErrorResponse::new("请求过于频繁，请稍后重试".to_string()));
    }
    Ok(())
}

/// 注册
pub async fn register(
    Extension(request_id): Extension<RequestId>,
    headers: HeaderMap,
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> Result<Json<ApiResponse<RegisterResult>>, ErrorResponse> {
    log_info(&request_id, "注册请求参数", &payload);
    let key = format!("rate_limit:auth:register:{}", client_ip(&headers));
    check_rate_limit(&state, &key, 3, 60).await?;

    let user_repo = UserRepository::new(state.pool.clone());
    let service = AuthService::new(
        user_repo,
        state.redis_client.clone(),
        state.config.auth.clone(),
    );

    match service.register(payload).await {
        Ok((user_model, access_token, refresh_token)) => {
            let ip = client_ip(&headers);
            let uid = user_model.id.clone();
            let audit_repo = AuditLogRepository::new(state.pool.clone());
            let _ = audit_repo
                .insert(Some(uid), "user.register", None, Some(ip), None)
                .await;
            let data = RegisterResult::from((user_model, access_token, refresh_token));
            let response = ApiResponse::success_with_message(data, "注册成功，已自动登录");
            log_info(&request_id, "注册成功", &response);
            Ok(Json(response))
        }
        Err(e) => {
            log_info(&request_id, "注册失败", &e.to_string());
            Err(ErrorResponse::new(format!("注册失败：{}", e)))
        }
    }
}

/// 登录
pub async fn login(
    Extension(request_id): Extension<RequestId>,
    headers: HeaderMap,
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<ApiResponse<LoginResult>>, ErrorResponse> {
    log_info(&request_id, "登录请求参数", &payload);
    let key = format!("rate_limit:auth:login:{}", client_ip(&headers));
    check_rate_limit(&state, &key, 5, 60).await?;

    let user_repo = UserRepository::new(state.pool.clone());
    let service = AuthService::new(
        user_repo,
        state.redis_client.clone(),
        state.config.auth.clone(),
    );

    match service.login(payload).await {
        Ok((user_model, access_token, refresh_token)) => {
            let ip = client_ip(&headers);
            let uid = user_model.id.clone();
            let audit_repo = AuditLogRepository::new(state.pool.clone());
            let _ = audit_repo
                .insert(Some(uid), "user.login", None, Some(ip), None)
                .await;
            let data = LoginResult::from((user_model, access_token, refresh_token));
            let response = ApiResponse::success_with_message(data, "登录成功");
            log_info(&request_id, "登录成功", &response);
            Ok(Json(response))
        }
        Err(e) => {
            log_info(&request_id, "登录失败", &e.to_string());
            Err(ErrorResponse::new(format!("登录失败：{}", e)))
        }
    }
}

/// 刷新 Token
pub async fn refresh(
    Extension(request_id): Extension<RequestId>,
    State(state): State<AppState>,
    Json(payload): Json<RefreshRequest>,
) -> Result<Json<ApiResponse<RefreshResult>>, ErrorResponse> {
    log_info(
        &request_id,
        "刷新 token 请求",
        &json!({"device_id": "default"}),
    );

    let user_repo = UserRepository::new(state.pool.clone());
    let service = AuthService::new(
        user_repo,
        state.redis_client.clone(),
        state.config.auth.clone(),
    );

    match service.refresh_access_token(&payload.refresh_token).await {
        Ok((access_token, refresh_token)) => {
            let data = RefreshResult {
                access_token,
                refresh_token,
            };
            let response = ApiResponse::success_with_message(data, "Token 刷新成功");

            log_info(&request_id, "刷新成功", &json!({"access_token": "***"}));
            Ok(Json(response))
        }
        Err(e) => {
            log_info(&request_id, "刷新失败", &e.to_string());
            Err(ErrorResponse::new(format!("Token 刷新失败：{}", e)))
        }
    }
}

/// 当前登录用户
pub async fn me(
    Extension(request_id): Extension<RequestId>,
    State(state): State<AppState>,
    Extension(user_id): Extension<String>,
) -> Result<Json<ApiResponse<UserProfile>>, ErrorResponse> {
    log_info(&request_id, "获取当前用户", &format!("user_id={}", user_id));

    let user_repo = UserRepository::new(state.pool.clone());
    match user_repo.find_by_id(&user_id).await {
        Ok(Some(user)) => Ok(Json(ApiResponse::success_with_message(
            UserProfile::from(user),
            "当前用户获取成功",
        ))),
        Ok(None) => Err(ErrorResponse::new("用户不存在".to_string())),
        Err(e) => Err(ErrorResponse::new(format!("当前用户获取失败：{}", e))),
    }
}

pub async fn update_profile(
    Extension(request_id): Extension<RequestId>,
    State(state): State<AppState>,
    Extension(user_id): Extension<String>,
    Json(payload): Json<UpdateProfileRequest>,
) -> Result<Json<ApiResponse<UserProfile>>, ErrorResponse> {
    log_info(&request_id, "更新个人资料", &format!("user_id={}", user_id));

    let user_repo = UserRepository::new(state.pool.clone());
    match user_repo
        .update_profile(
            &user_id,
            payload.username,
            payload.nickname,
            payload.bio,
        )
        .await
    {
        Ok(user) => Ok(Json(ApiResponse::success_with_message(
            UserProfile::from(user),
            "个人资料更新成功",
        ))),
        Err(_) => Err(ErrorResponse::new("个人资料更新失败，请稍后重试")),
    }
}

pub async fn upload_avatar(
    State(state): State<AppState>,
    Extension(user_id): Extension<String>,
    mut multipart: Multipart,
) -> Result<Json<ApiResponse<UserProfile>>, ErrorResponse> {
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| ErrorResponse::new("头像上传失败，请重新选择文件"))?
    {
        if field.name() != Some("avatar") {
            continue;
        }

        let ext = avatar_extension(field.content_type())
            .ok_or_else(|| ErrorResponse::new("仅支持 JPG、PNG、WEBP 头像"))?;
        let bytes = field
            .bytes()
            .await
            .map_err(|_| ErrorResponse::new("头像上传失败，请重新选择文件"))?;
        if bytes.len() > 2 * 1024 * 1024 {
            return Err(ErrorResponse::new("头像文件不能超过 2MB"));
        }

        tokio::fs::create_dir_all("data/avatar").await.ok();
        let filename = format!("{}-{}.{}", user_id, Uuid::new_v4(), ext);
        let path = format!("data/avatar/{}", filename);
        tokio::fs::write(&path, bytes)
            .await
            .map_err(|_| ErrorResponse::new("头像保存失败，请稍后重试"))?;
        let avatar_url = format!("/api/avatar/{}", filename);
        let user = UserRepository::new(state.pool.clone())
            .update_avatar(&user_id, avatar_url)
            .await
            .map_err(|_| ErrorResponse::new("头像更新失败，请稍后重试"))?;
        return Ok(Json(ApiResponse::success_with_message(
            UserProfile::from(user),
            "头像上传成功",
        )));
    }

    Err(ErrorResponse::new("请选择要上传的头像文件"))
}

/// 删除账号
pub async fn delete_account(
    Extension(request_id): Extension<RequestId>,
    State(state): State<AppState>,
    Extension(user_id): Extension<String>,
    Json(payload): Json<DeleteUserRequest>,
) -> Result<Json<ApiResponse<()>>, ErrorResponse> {
    log_info(&request_id, "删除账号请求", &format!("user_id={}", user_id));

    let user_repo = UserRepository::new(state.pool.clone());
    let service = AuthService::new(
        user_repo,
        state.redis_client.clone(),
        state.config.auth.clone(),
    );

    let delete_request = DeleteUserRequest {
        user_id: user_id.clone(),
        password: payload.password,
    };

    match service.delete_user(delete_request).await {
        Ok(_) => {
            let audit_repo = AuditLogRepository::new(state.pool.clone());
            let _ = audit_repo
                .insert(
                    Some(user_id.clone()),
                    "user.delete_account",
                    None,
                    None,
                    None,
                )
                .await;
            log_info(&request_id, "账号删除成功", &format!("user_id={}", user_id));
            let response = ApiResponse::success_with_message((), "账号删除成功");
            Ok(Json(response))
        }
        Err(e) => {
            log_info(&request_id, "账号删除失败", &e.to_string());
            Err(ErrorResponse::new(format!("账号删除失败：{}", e)))
        }
    }
}

/// 刷新令牌
pub async fn delete_refresh_token(
    Extension(request_id): Extension<RequestId>,
    State(state): State<AppState>,
    Extension(user_id): Extension<String>,
) -> Result<Json<ApiResponse<()>>, ErrorResponse> {
    log_info(
        &request_id,
        "删除刷新令牌请求",
        &format!("user_id={}", user_id),
    );

    let user_repo = UserRepository::new(state.pool.clone());
    let service = AuthService::new(
        user_repo,
        state.redis_client.clone(),
        state.config.auth.clone(),
    );

    match service.delete_refresh_token(&user_id).await {
        Ok(_) => {
            log_info(
                &request_id,
                "刷新令牌删除成功",
                &format!("user_id={}", user_id),
            );
            let response = ApiResponse::success_with_message((), "已退出登录");
            Ok(Json(response))
        }
        Err(e) => {
            log_info(&request_id, "刷新令牌删除失败", &e.to_string());
            Err(ErrorResponse::new(format!("退出登录失败：{}", e)))
        }
    }
}
