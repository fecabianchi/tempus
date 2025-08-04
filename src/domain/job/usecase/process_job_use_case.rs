use crate::config::app_config::AppConfig;
use crate::domain::job::entity::job_entity::JobEntity;
use crate::domain::job::entity::job_metadata_entity::JobMetadataEntity;
use crate::domain::job::r#enum::job_enum::{JobMetadataStatus, JobType};
use crate::domain::job::port::driven::job_metadata_repository_port::JobMetadataRepositoryPort;
use crate::domain::job::port::driven::job_repository_port::JobRepositoryPort;
use crate::domain::job::port::driver::process_job_use_case_port::ProcessJobUseCasePort;
use crate::error::{Result, TempusError};
use chrono::{NaiveDateTime, Utc};
use log::{error, info, warn};
use once_cell::sync::Lazy;
use reqwest::{Client, Response};
use sea_orm::JsonValue;
use std::sync::Arc;

pub struct ProcessJobUseCase<
    JR: JobRepositoryPort + Send + Sync,
    JMR: JobMetadataRepositoryPort + Send + Sync,
> {
    job_repository: JR,
    job_metadata_repository: JMR,
    config: AppConfig,
}

fn create_http_client(config: &AppConfig) -> Result<Client> {
    Client::builder()
        .pool_idle_timeout(config.http.pool_idle_timeout())
        .timeout(config.http.request_timeout())
        .build()
        .map_err(TempusError::Http)
}

static HTTP_CLIENT: Lazy<Result<Client>> = Lazy::new(|| {
    AppConfig::load()
        .and_then(|config| create_http_client(&config))
});

impl<JR: JobRepositoryPort + Send + Sync, JMR: JobMetadataRepositoryPort + Send + Sync>
    ProcessJobUseCase<JR, JMR>
{
    pub fn new(job_repository: JR, job_metadata_repository: JMR, config: &AppConfig) -> Self {
        Self {
            job_repository,
            job_metadata_repository,
            config: config.clone(),
        }
    }
}

async fn handle_success<JMR>(metadata: JobMetadataEntity, job_metadata_repository: JMR) -> Result<()>
where
    JMR: JobMetadataRepositoryPort + Send + Sync + 'static,
{
    let updated_metadata = JobMetadataEntity {
        job_id: metadata.job_id,
        status: JobMetadataStatus::Completed,
        failure: metadata.failure,
        processed_at: Some(Utc::now().naive_utc()),
    };

    job_metadata_repository
        .update_status(updated_metadata)
        .await
        .map_err(TempusError::Database)
}

fn should_retry(retries: i32, max_retries: i32) -> bool {
    retries < max_retries
}

fn backoff(time: NaiveDateTime, retries: i32, base_delay_minutes: u32) -> NaiveDateTime {
    let delay_minutes = base_delay_minutes * (2u32.pow(retries as u32));
    time + chrono::Duration::minutes(delay_minutes as i64)
}

async fn handle_failure<JR, JMR>(
    job: JobEntity,
    job_metadata: JobMetadataEntity,
    job_repository: JR,
    job_metadata_repository: JMR,
    error_msg: String,
    config: &AppConfig,
) -> Result<()>
where
    JR: JobRepositoryPort + Send + Sync,
    JMR: JobMetadataRepositoryPort + Send + Sync,
{
    let current_retries = job.retries;
    if should_retry(current_retries, config.engine.retry_attempts) {
        info!("Retrying job {} (attempt {}/{})", job.id, current_retries + 1, config.engine.retry_attempts);
        
        let new_time = backoff(job.time, current_retries + 1, config.engine.base_delay_minutes);
        let retry_metadata = JobMetadataEntity {
            job_id: job_metadata.job_id,
            status: JobMetadataStatus::Scheduled,
            failure: None,
            processed_at: None,
        };

        job_repository
            .handle_retry_transaction(job.id, new_time, retry_metadata)
            .await
            .map_err(TempusError::Database)?;
    } else {
        warn!("Job {} failed permanently after {} attempts: {}", job.id, current_retries, error_msg);
        
        let failed_metadata = JobMetadataEntity {
            job_id: job_metadata.job_id,
            failure: Some(error_msg),
            processed_at: None,
            status: JobMetadataStatus::Failed,
        };

        job_metadata_repository
            .update_status(failed_metadata)
            .await
            .map_err(TempusError::Database)?;
    }
    
    Ok(())
}

impl<JR, JMR> ProcessJobUseCasePort for ProcessJobUseCase<JR, JMR>
where
    JR: JobRepositoryPort + Send + Sync + Clone + 'static,
    JMR: JobMetadataRepositoryPort + Send + Sync + Clone + 'static,
{
    async fn execute(&self) -> Result<()> {
        let jobs = self
            .job_repository
            .find_and_flag_processing()
            .await
            .map_err(TempusError::Database)?;

        if jobs.is_empty() {
            return Ok(());
        }

        info!("Processing {} jobs", jobs.len());
        let semaphore = Arc::new(tokio::sync::Semaphore::new(self.config.engine.max_concurrent_jobs));

        let mut handles = Vec::new();

        for job in jobs {
            let permit = match semaphore.clone().acquire_owned().await {
                Ok(permit) => permit,
                Err(e) => {
                    error!("Failed to acquire semaphore permit: {}", e);
                    continue;
                }
            };
            
            let job_repository = self.job_repository.clone();
            let job_metadata_repository = self.job_metadata_repository.clone();
            let job_target = job.target.clone();
            let job_payload = job.payload.clone();
            let inner_job = job.clone();
            let config = self.config.clone();

            let handle = tokio::spawn(async move {
                let _permit = permit;
                
                let result = match job.r#type {
                    JobType::Http => match job.metadata {
                        None => {
                            warn!("Metadata is missing for jobId: {}", &job.id);
                            Err(TempusError::JobProcessing("Missing job metadata".to_string()))
                        }
                        Some(metadata) => {
                            match perform_request(job_target, job_payload).await {
                                Ok(_) => {
                                    info!("Job {} completed successfully", job.id);
                                    handle_success(metadata, job_metadata_repository).await
                                }
                                Err(e) => {
                                    error!("Job {} failed: {}", job.id, e);
                                    handle_failure(
                                        inner_job,
                                        metadata,
                                        job_repository,
                                        job_metadata_repository,
                                        e.to_string(),
                                        &config,
                                    ).await
                                }
                            }
                        }
                    },
                };
                
                if let Err(e) = result {
                    error!("Error processing job {}: {:?}", job.id, e);
                }
            });
            
            handles.push(handle);
        }

        for handle in handles {
            if let Err(e) = handle.await {
                error!("Task join error: {}", e);
            }
        }

        Ok(())
    }
}

fn validate_url(url: &str) -> Result<()> {
    if url.is_empty() {
        return Err(TempusError::Validation("URL cannot be empty".to_string()));
    }
    
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err(TempusError::Validation("URL must start with http:// or https://".to_string()));
    }
    
    Ok(())
}

async fn perform_request(target: String, payload: JsonValue) -> Result<Response> {
    validate_url(&target)?;
    
    let client = HTTP_CLIENT.as_ref()
        .map_err(|e| TempusError::Config(format!("Failed to initialize HTTP client: {}", e)))?;
    
    client
        .post(target)
        .json(&payload)
        .send()
        .await
        .map_err(TempusError::Http)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_retry() {
        assert!(should_retry(0, 3));
        assert!(should_retry(2, 3));
        assert!(!should_retry(3, 3));
        assert!(!should_retry(5, 3));
    }

    #[test]
    fn test_backoff_calculation() {
        let base_time = chrono::DateTime::from_timestamp(1000, 0).unwrap().naive_utc();
        
        let result1 = backoff(base_time, 0, 2);
        let expected1 = base_time + chrono::Duration::minutes(2);
        assert_eq!(result1, expected1);

        let result2 = backoff(base_time, 1, 2);
        let expected2 = base_time + chrono::Duration::minutes(4);
        assert_eq!(result2, expected2);

        let result3 = backoff(base_time, 2, 2);
        let expected3 = base_time + chrono::Duration::minutes(8);
        assert_eq!(result3, expected3);
    }

    #[test]
    fn test_validate_url() {
        assert!(validate_url("https://example.com").is_ok());
        assert!(validate_url("http://example.com").is_ok());
        assert!(validate_url("").is_err());
        assert!(validate_url("ftp://example.com").is_err());
        assert!(validate_url("invalid-url").is_err());
    }
}