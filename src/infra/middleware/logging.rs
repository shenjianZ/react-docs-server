use axum::{extract::Request, response::Response};
use std::time::Instant;

/// Request ID 标记
#[derive(Clone)]
pub struct RequestId(pub String);

/// 请求日志中间件
pub async fn request_logging_middleware(
    mut req: Request,
    next: axum::middleware::Next,
) -> Response {
    let start = Instant::now();

    // 提取请求信息
    let method = req.method().clone();
    let path = req.uri().path().to_string();
    let query = req.uri().query().map(|s| s.to_string());

    // 生成请求 ID
    let request_id = uuid::Uuid::new_v4().to_string();

    // 将 request_id 存储到请求扩展中
    req.extensions_mut().insert(RequestId(request_id.clone()));

    // 第1条日志：请求开始
    let separator = "=".repeat(80);
    let header = format!("{} {}", method, path);

    tracing::info!("{}", separator);
    tracing::info!("{}", header);
    tracing::info!("{}", separator);

    let now_beijing = chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
    let query_str = query.as_deref().unwrap_or("无");
    tracing::info!(
        "[{}] 📥 查询参数: {} | 时间: {}",
        request_id,
        query_str,
        now_beijing
    );

    // 调用下一个处理器
    let response = next.run(req).await;

    // 第3条日志：请求完成
    let duration = start.elapsed();
    let status = response.status();
    tracing::info!(
        "[{}] ✅ 状态码: {} | 耗时: {}ms",
        request_id,
        status.as_u16(),
        duration.as_millis()
    );

    tracing::info!("{}", separator);

    response
}

/// 请求日志辅助工具
pub fn log_info<T: std::fmt::Debug>(request_id: &RequestId, label: &str, data: T) {
    let data_str = format!("{:?}", data);
    let truncated = if data_str.len() > 300 {
        format!("{}...", &data_str[..300])
    } else {
        data_str
    };

    tracing::info!("[{}] 🔧 {} | {}", request_id.0, label, truncated);
}
