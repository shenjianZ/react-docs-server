use crate::error::ErrorResponse;
use crate::infra::redis::redis_key::{BusinessType, RedisKey};
use crate::repositories::ip_blacklist_repository::IpBlacklistRepository;
use crate::AppState;
use axum::{
    extract::{Request, State},
    http::HeaderMap,
    middleware::Next,
    response::Response,
};

fn extract_ip(headers: &HeaderMap) -> String {
    headers
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.split(',').next())
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| "unknown".to_string())
}

/// IP 黑名单检查中间件
pub async fn ip_blacklist_middleware(
    State(state): State<AppState>,
    headers: HeaderMap,
    req: Request,
    next: Next,
) -> Result<Response, ErrorResponse> {
    let ip = extract_ip(&headers);
    if ip == "unknown" {
        return Ok(next.run(req).await);
    }

    // 先查 Redis 缓存
    let cache_key = RedisKey::new(BusinessType::Cache)
        .add_identifier("ip_blacklist")
        .add_identifier(&ip);
    if let Ok(Some(cached)) = state.redis_client.get_key(&cache_key).await {
        if cached == "1" {
            return Err(ErrorResponse::forbidden("当前 IP 已被限制访问"));
        }
    } else {
        // Redis miss，查数据库
        let repo = IpBlacklistRepository::new(state.pool.clone());
        if let Ok(true) = repo.is_blocked(&ip).await {
            let _ = state.redis_client.set_key_ex(&cache_key, "1", 300).await;
            return Err(ErrorResponse::forbidden("当前 IP 已被限制访问"));
        }
        // 缓存"未被封禁"状态，60 秒过期
        let _ = state.redis_client.set_key_ex(&cache_key, "0", 60).await;
    }

    Ok(next.run(req).await)
}
