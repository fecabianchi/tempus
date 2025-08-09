use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
};
use log::error;
use validator::Validate;

use crate::api::dto::{CreateJobRequest, CreateJobResponse, ApiError};
use crate::domain::job::usecase::create_job_use_case::{CreateJobUseCase, CreateJobRequest as DomainCreateJobRequest};
use crate::error::TempusError;
use crate::infrastructure::persistence::job::job_repository::JobRepository;

pub async fn create_job(
    State(job_repository): State<JobRepository>,
    Json(payload): Json<CreateJobRequest>,
) -> Result<Json<CreateJobResponse>, (StatusCode, Json<ApiError>)> {
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

    let domain_request = DomainCreateJobRequest {
        target: payload.target,
        time: payload.time,
        job_type: payload.job_type,
        payload: payload.payload,
    };

    let create_job_use_case = CreateJobUseCase::new(job_repository);
    
    match create_job_use_case.execute(domain_request).await {
        Ok(domain_response) => {
            Ok(Json(CreateJobResponse {
                id: domain_response.id,
                message: domain_response.message,
            }))
        }
        Err(TempusError::Validation(msg)) => {
            error!("Validation error: {}", msg);
            Err((
                StatusCode::BAD_REQUEST,
                Json(ApiError::bad_request(msg)),
            ))
        }
        Err(TempusError::Database(db_err)) => {
            error!("Database error: {:?}", db_err);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError::internal_error("Failed to create job")),
            ))
        }
        Err(e) => {
            error!("Unexpected error: {:?}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError::internal_error("Failed to create job")),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::extract::State;
    use chrono::Utc;
    use sea_orm::JsonValue;
    use uuid::Uuid;

    #[test]
    fn test_request_conversion() {
        let api_request = CreateJobRequest {
            target: "https://example.com".to_string(),
            time: Some(Utc::now().naive_utc()),
            job_type: "http".to_string(),
            payload: JsonValue::Null,
        };
        
        let domain_request = DomainCreateJobRequest {
            target: api_request.target.clone(),
            time: api_request.time,
            job_type: api_request.job_type.clone(),
            payload: api_request.payload.clone(),
        };
        
        assert_eq!(domain_request.target, api_request.target);
        assert_eq!(domain_request.job_type, api_request.job_type);
    }
}