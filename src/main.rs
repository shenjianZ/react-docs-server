mod cli;
mod config;
mod db;
mod domain;
mod error;
mod handlers;
mod infra;
mod repositories;
mod services;
mod utils;

use axum::{
    http::HeaderValue,
    routing::{delete, get, post, put},
    Router,
};
use clap::Parser;
use cli::CliArgs;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// 应用状态
#[derive(Clone)]
pub struct AppState {
    pub pool: db::DbPool,
    pub config: config::app::AppConfig,
    pub redis_client: infra::redis::redis_client::RedisClient,
}

fn cors_layer(server: &config::server::ServerConfig) -> CorsLayer {
    let allow_any_origin = server
        .cors_allowed_origins
        .iter()
        .any(|origin| origin == "*");

    let layer = CorsLayer::new().allow_methods(Any).allow_headers(Any);

    if allow_any_origin {
        return layer.allow_origin(Any);
    }

    let origins: Vec<HeaderValue> = server
        .cors_allowed_origins
        .iter()
        .filter_map(|origin| origin.parse::<HeaderValue>().ok())
        .collect();

    layer.allow_origin(origins)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 解析命令行参数
    let args = CliArgs::parse();

    // 初始化日志
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| args.get_log_filter().into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // 打印启动信息
    args.print_startup_info();

    // 设置工作目录（如果指定）
    if let Some(ref work_dir) = args.work_dir {
        std::env::set_current_dir(work_dir).ok();
        println!("Working directory set to: {}", work_dir.display());
    }

    // 解析配置文件路径（可选）
    let config_path = args.resolve_config_path();

    // 加载配置（支持 CLI 覆盖）
    // 如果没有配置文件，将仅使用环境变量和默认值
    let config = config::app::AppConfig::load_with_overrides(
        config_path,
        args.get_overrides(),
        args.env.as_str(),
    )?;

    tracing::info!("Configuration loaded successfully");
    tracing::info!("Environment: {}", args.env.as_str());
    tracing::info!("Debug mode: {}", args.is_debug_enabled());

    // 初始化数据库（自动创建数据库和表）
    let pool = db::init_database(&config.database).await?;

    // 初始化 Redis 客户端
    let redis_client = infra::redis::redis_client::RedisClient::new(&config.redis.build_url())
        .await
        .map_err(|e| anyhow::anyhow!("Redis 初始化失败: {}", e))?;

    tracing::info!("Redis 连接池初始化成功");

    // 创建应用状态
    let app_state = AppState {
        pool: pool.clone(),
        config: config.clone(),
        redis_client,
    };

    // ========== 公开路由 ==========
    let public_routes = Router::new()
        .route("/health", get(handlers::health::health_check))
        .route("/info", get(handlers::health::server_info))
        .route("/auth/register", post(handlers::auth::register))
        .route("/auth/login", post(handlers::auth::login))
        .route("/auth/refresh", post(handlers::auth::refresh))
        .route("/analytics/view", post(handlers::analytics::track_view))
        .route(
            "/analytics/duration",
            post(handlers::analytics::track_duration),
        )
        .route(
            "/analytics/popular",
            get(handlers::analytics::popular_pages),
        )
        .route("/analytics/overview", get(handlers::analytics::overview))
        .route("/analytics/trends", get(handlers::analytics::trends))
        .route("/feedback/status", get(handlers::feedback::feedback_status))
        .route("/feedback", post(handlers::feedback::create_feedback))
        .route("/comments", get(handlers::comment::list_comments));

    // ========== 受保护路由 ==========
    let protected_routes = Router::new()
        .route("/auth/me", get(handlers::auth::me))
        .route("/auth/profile", put(handlers::auth::update_profile))
        .route("/auth/avatar", post(handlers::auth::upload_avatar))
        .route("/bookmarks", get(handlers::bookmark::list_bookmarks))
        .route("/bookmarks", post(handlers::bookmark::create_bookmark))
        .route("/bookmarks/check", get(handlers::bookmark::check_bookmark))
        .route(
            "/bookmarks/:id",
            delete(handlers::bookmark::delete_bookmark),
        )
        .route("/comments", post(handlers::comment::create_comment))
        .route("/comments/:id", put(handlers::comment::update_comment))
        .route("/comments/:id", delete(handlers::comment::delete_comment))
        .route("/comments/:id/like", post(handlers::comment::like_comment))
        .route("/auth/delete", post(handlers::auth::delete_account))
        .route(
            "/auth/delete-refresh-token",
            post(handlers::auth::delete_refresh_token),
        )
        // JWT 认证中间件（仅应用于受保护路由）
        .route_layer(axum::middleware::from_fn_with_state(
            app_state.clone(),
            infra::middleware::auth::auth_middleware,
        ));

    // ========== 合并路由 ==========
    let api_routes = public_routes.merge(protected_routes);
    let cors = cors_layer(&config.server);

    let app = Router::new()
        .nest("/api", api_routes)
        .nest_service("/api/avatar", ServeDir::new("data/avatar"))
        // CORS（应用于所有路由）
        .layer(cors)
        // IP 黑名单中间件
        .layer(axum::middleware::from_fn_with_state(
            app_state.clone(),
            infra::middleware::ip_blacklist::ip_blacklist_middleware,
        ))
        // 日志中间件（应用于所有路由）
        .layer(axum::middleware::from_fn_with_state(
            app_state.clone(),
            infra::middleware::logging::request_logging_middleware,
        ))
        .with_state(app_state);

    // 启动服务器
    let addr = format!("{}:{}", config.server.host, config.server.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("Server listening on {}", addr);
    tracing::info!("Press Ctrl+C to stop");

    axum::serve(listener, app).await?;

    Ok(())
}
