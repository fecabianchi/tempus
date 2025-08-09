use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use log::error;
use uuid::Uuid;

use crate::api::dto::ApiError;
use crate::domain::job::usecase::delete_job_use_case::DeleteJobUseCase;
use crate::error::TempusError;
use crate::infrastructure::persistence::job::job_repository::JobRepository;

pub async fn delete_job(
    State(job_repository): State<JobRepository>,
    Path(job_id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, Json<ApiError>)> {
    let delete_job_use_case = DeleteJobUseCase::new(job_repository);
    
    match delete_job_use_case.execute(job_id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(TempusError::Validation(msg)) => {
            error!("Validation error: {}", msg);
            Err((
                StatusCode::NOT_FOUND,
                Json(ApiError::not_found(msg)),
            ))
        }
        Err(TempusError::Database(db_err)) => {
            error!("Database error while deleting job {}: {:?}", job_id, db_err);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError::internal_error("Failed to delete job")),
            ))
        }
        Err(e) => {
            error!("Unexpected error while deleting job {}: {:?}", job_id, e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError::internal_error("Failed to delete job")),
            ))
        }
    }
}