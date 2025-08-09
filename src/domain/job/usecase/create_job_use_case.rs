use chrono::{NaiveDateTime, Utc};
use log::info;
use uuid::Uuid;

use crate::domain::job::entity::job_entity::JobEntity;
use crate::domain::job::r#enum::job_enum::JobType;
use crate::domain::job::port::driven::job_repository_port::JobRepositoryPort;
use crate::error::{TempusError, Result};

pub struct CreateJobUseCase<R: JobRepositoryPort> {
    job_repository: R,
}

impl<R: JobRepositoryPort> CreateJobUseCase<R> {
    pub fn new(job_repository: R) -> Self {
        Self { job_repository }
    }

    pub async fn execute(&self, request: CreateJobRequest) -> Result<CreateJobResponse> {
        let job_type = self.parse_job_type(&request.job_type)?;
        let job_id = Uuid::new_v4();
        let scheduled_time = request.time.unwrap_or_else(|| Utc::now().naive_utc());

        let job_entity = JobEntity {
            id: job_id,
            time: scheduled_time,
            target: request.target,
            retries: 0,
            r#type: job_type,
            payload: request.payload,
            metadata: None,
        };

        self.job_repository.save(&job_entity).await
            .map_err(TempusError::from)?;

        info!("Job created successfully with ID: {}", job_id);

        Ok(CreateJobResponse {
            id: job_id,
            message: "Job created successfully".to_string(),
        })
    }

    fn parse_job_type(&self, job_type_str: &str) -> Result<JobType> {
        match job_type_str.to_lowercase().as_str() {
            "kafka" => Ok(JobType::Kafka),
            "http" => Ok(JobType::Http),
            _ => Err(TempusError::Validation(format!(
                "Invalid job type: {}. Supported types: http", 
                job_type_str
            )))
        }
    }
}

#[derive(Debug)]
pub struct CreateJobRequest {
    pub target: String,
    pub time: Option<NaiveDateTime>,
    pub job_type: String,
    pub payload: sea_orm::JsonValue,
}

#[derive(Debug)]
pub struct CreateJobResponse {
    pub id: Uuid,
    pub message: String,
}