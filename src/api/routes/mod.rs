pub mod health;
pub mod job_routes;

use axum::Router;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

use crate::infrastructure::persistence::job::job_repository::JobRepository;

pub fn create_router(job_repository: JobRepository) -> Router {
    let health_router = health::health_router();
    
    let job_router = job_routes::job_router()
        .with_state(job_repository);

    Router::new()
        .merge(health_router)
        .merge(job_router)
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
}