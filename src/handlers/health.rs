use crate::db;
use crate::AppState;
use axum::{
    extract::State,
    response::{IntoResponse, Json},
};
use serde_json::json;

/// 健康检查端点
pub async fn health_check(State(state): State<AppState>) -> impl IntoResponse {
    let database_ok = db::health_check(&state.pool).await.is_ok();
    let redis_ok = state.redis_client.ping().await.is_ok();
    let disk_ok = std::env::current_dir()
        .and_then(std::fs::metadata)
        .map(|metadata| metadata.is_dir())
        .unwrap_or(false);
    let status = if database_ok && redis_ok && disk_ok {
        "ok"
    } else {
        "unavailable"
    };

    Json(json!({
        "status": status,
        "message": if status == "ok" { "服务运行正常" } else { "服务不可用，请检查后端依赖" },
        "checks": {
            "database": database_ok,
            "redis": redis_ok,
            "disk": disk_ok
        }
    }))
}

/// 获取服务器信息
pub async fn server_info() -> impl IntoResponse {
    Json(json!({
        "name": "web-rust-template",
        "version": "0.1.0",
        "status": "running",
        "message": "服务信息获取成功",
        "timestamp": chrono::Utc::now().timestamp()
    }))
}
