use axum::{
    http::StatusCode,
    response::Response,
    routing::get,
    Router,
};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use crate::infrastructure::metrics::get_prometheus_handle;
use log::info;

fn create_metrics_response(metrics: String) -> Response {
    Response::builder()
        .header("content-type", "text/plain; charset=utf-8")
        .body(metrics.into())
        .unwrap()
}

async fn metrics_handler() -> Result<Response, StatusCode> {
    get_prometheus_handle()
        .map(|handle| handle.render())
        .map(create_metrics_response)
        .ok_or_else(|| {
            log::error!("Prometheus handle not initialized");
            StatusCode::INTERNAL_SERVER_ERROR
        })
}

async fn health_handler() -> &'static str {
    "OK"
}

fn create_router() -> Router {
    Router::new()
        .route("/metrics", get(metrics_handler))
        .route("/health", get(health_handler))
}

fn create_socket_address(port: u16) -> SocketAddr {
    SocketAddr::from(([0, 0, 0, 0], port))
}

fn log_server_info(addr: SocketAddr) {
    info!("Engine metrics server listening on {}", addr);
    info!("Engine metrics endpoint available at http://{}/metrics", addr);
}

pub async fn start_metrics_server(port: u16) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let app = create_router();
    let addr = create_socket_address(port);
    let listener = TcpListener::bind(addr).await?;

    log_server_info(addr);
    axum::serve(listener, app).await?;

    Ok(())
}