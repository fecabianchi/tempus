use axum::{
    routing::get,
    Router,
};

use crate::api::handlers;

pub fn health_router() -> Router {
    Router::new()
        .route("/health", get(handlers::health_check))
}