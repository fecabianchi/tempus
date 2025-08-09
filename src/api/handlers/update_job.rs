use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use log::error;
use uuid::Uuid;
use validator::Validate;

use crate::api::dto::{ApiError, UpdateJobTimeRequest};
use crate::domain::job::usecase::update_job_time_use_case::UpdateJobTimeUseCase;
use crate::error::TempusError;
use crate::infrastructure::persistence::job::job_repository::JobRepository;

pub async fn update_job_time(
    State(job_repository): State<JobRepository>,
    Path(job_id): Path<Uuid>,
    Json(payload): Json<UpdateJobTimeRequest>,
) -> Result<StatusCode, (StatusCode, Json<ApiError>)> {
    if let Err(validation_errors) = payload.validate() {
        error!("Validation failed: {:?}", validation_errors);
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError::validation_error(format!(
                "Validation errors: {:?}", 
                validation_errors
            ))),
        ));
    }

    let update_job_time_use_case = UpdateJobTimeUseCase::new(job_repository);
    
    match update_job_time_use_case.execute(job_id, payload.time).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(TempusError::Validation(msg)) => {
            error!("Validation error: {}", msg);
            Err((
                StatusCode::NOT_FOUND,
                Json(ApiError::not_found(msg)),
            ))
        }
        Err(TempusError::Database(db_err)) => {
            error!("Database error while updating job {}: {:?}", job_id, db_err);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError::internal_error("Failed to update job")),
            ))
        }
        Err(e) => {
            error!("Unexpected error while updating job {}: {:?}", job_id, e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError::internal_error("Failed to update job")),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_update_request_validation() {
        let valid_request = UpdateJobTimeRequest {
            time: Utc::now().naive_utc(),
        };
        
        assert!(valid_request.validate().is_ok());
    }
}