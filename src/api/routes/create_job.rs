use axum::{
    routing::post,
    Router,
};

use crate::api::handlers;
use crate::infrastructure::persistence::job::job_repository::JobRepository;

/// Creates the job-related router
pub fn job_router() -> Router<JobRepository> {
    Router::new()
        .route("/jobs", post(handlers::create_job))
}