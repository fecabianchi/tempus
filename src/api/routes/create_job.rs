use axum::{
    routing::post,
    Router,
};

use crate::api::handlers;
use crate::infrastructure::persistence::job::job_repository::JobRepository;

pub fn job_router() -> Router<JobRepository> {
    Router::new()
        .route("/jobs", post(handlers::create_job))
}