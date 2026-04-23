use crate::error::ErrorResponse;
use crate::AppState;
use axum::{
    extract::{Request, State},
    http::HeaderMap,
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Claims {
    pub sub: String, // user_id
    #[allow(dead_code)]
    pub exp: usize,
}

/// JWT 认证中间件
pub async fn auth_middleware(
    State(state): State<AppState>,
    headers: HeaderMap,
    mut req: Request,
    next: Next,
) -> Result<Response, ErrorResponse> {
    // 1. 提取 Authorization header
    let auth_header = headers
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| ErrorResponse::unauthorized("请先登录"))?;

    if !auth_header.starts_with("Bearer ") {
        return Err(ErrorResponse::unauthorized("登录状态无效，请重新登录"));
    }

    let token = &auth_header[7..];

    // 2. 验证 JWT
    let jwt_secret = &state.config.auth.jwt_secret;

    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(jwt_secret.as_ref()),
        &Validation::default(),
    )
    .map_err(|_| ErrorResponse::unauthorized("登录状态已过期，请重新登录"))?;

    // 3. 将 user_id 添加到请求扩展
    req.extensions_mut().insert(token_data.claims.sub);

    Ok(next.run(req).await)
}
