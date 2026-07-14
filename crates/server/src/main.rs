//! Interface 层 —— Axum HTTP 服务入口。

mod dto;
mod mock_service;
mod real_service;
mod routes;
mod service;

use std::net::SocketAddr;
use std::sync::Arc;

use tracing::info;

use crate::mock_service::{MockBuildService, MockConfigService};
use crate::real_service::{RealBuildService, RealConfigService};
use crate::routes::AppState;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .init();

    // 检查是否使用 Real Service
    let use_real = std::env::var("USE_REAL_SERVICE")
        .ok()
        .and_then(|v| v.parse::<bool>().ok())
        .unwrap_or(false);

    let state = if use_real {
        info!("使用 Real Service（生产模式）");

        // 加载配置
        let config_dir = std::env::var("CONFIG_DIR").unwrap_or_else(|_| "config".to_string());
        info!("加载配置: {}", config_dir);
        let config = config_compiler::load_and_compile(std::path::Path::new(&config_dir))?;
        info!("配置加载成功，字段数量: {}", config.schema.fields.len());

        // 加载模板
        let template_path =
            std::env::var("TEMPLATE_PATH").unwrap_or_else(|_| "template.imazingapp".to_string());
        info!("加载模板: {}", template_path);
        let template_bytes = std::fs::read(&template_path)?;
        info!("模板加载成功，大小: {} 字节", template_bytes.len());

        AppState {
            config_service: Arc::new(RealConfigService::new(config.clone())),
            build_service: Arc::new(RealBuildService::new(config.clone(), template_bytes)),
        }
    } else {
        info!("使用 Mock Service（开发模式）");

        AppState {
            config_service: Arc::new(MockConfigService),
            build_service: Arc::new(MockBuildService),
        }
    };

    // 限流已移除：请求经微信云托管内网直达容器，来源 IP 为网关 IP，
    // IP 限流会误伤全体用户。防刷改由小程序端冷却兜底。
    let app = routes::router(state);

    // 从环境变量读取端口，默认 80（云托管）或 8080（本地开发）
    let port = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(8080);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!(%addr, "imazing-server 启动中");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
