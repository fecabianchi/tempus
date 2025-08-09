use axum::{
    routing::{delete, patch, post},
    Router,
};

use crate::api::handlers;
use crate::infrastructure::persistence::job::job_repository::JobRepository;

pub fn job_router() -> Router<JobRepository> {
    Router::new()
        .route("/jobs", post(handlers::create_job))
        .route("/jobs/:job_id", delete(handlers::delete_job))
        .route("/jobs/:job_id/time", patch(handlers::update_job_time))
}