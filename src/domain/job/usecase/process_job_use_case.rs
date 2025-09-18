use crate::config::app_config::AppConfig;
use crate::domain::job::entity::job_entity::JobEntity;
use crate::domain::job::entity::job_metadata_entity::JobMetadataEntity;
use crate::domain::job::r#enum::job_enum::{JobMetadataStatus, JobType};
use crate::domain::job::port::driven::job_metadata_repository_port::JobMetadataRepositoryPort;
use crate::domain::job::port::driven::job_repository_port::JobRepositoryPort;
use crate::domain::job::port::driver::process_job_use_case_port::ProcessJobUseCasePort;
use crate::error::{Result, TempusError};
use crate::infrastructure::kafka::kafka_publisher::publish_kafka_message;
use crate::infrastructure::metrics::{increment_jobs_processed, observe_job_duration, increment_http_requests, increment_kafka_messages, increment_current_processing_jobs, decrement_current_processing_jobs};
use chrono::{NaiveDateTime, Utc};
use std::time::Instant;
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

fn create_success_metadata(metadata: JobMetadataEntity) -> JobMetadataEntity {
    JobMetadataEntity {
        job_id: metadata.job_id,
        status: JobMetadataStatus::Completed,
        failure: metadata.failure,
        processed_at: Some(Utc::now().naive_utc()),
    }
}

async fn handle_success<JMR>(metadata: JobMetadataEntity, job_metadata_repository: JMR) -> Result<()>
where
    JMR: JobMetadataRepositoryPort + Send + Sync + 'static,
{
    let updated_metadata = create_success_metadata(metadata);

    job_metadata_repository
        .update_status(updated_metadata)
        .await
        .map_err(TempusError::Database)
        .map(|_| increment_jobs_processed("success"))
}

fn should_retry(retries: i32, max_retries: i32) -> bool {
    retries < max_retries
}

fn calculate_delay_minutes(retries: i32, base_delay_minutes: u32) -> u32 {
    base_delay_minutes * (2u32.pow(retries as u32))
}

fn backoff(time: NaiveDateTime, retries: i32, base_delay_minutes: u32) -> NaiveDateTime {
    let delay_minutes = calculate_delay_minutes(retries, base_delay_minutes);
    time + chrono::Duration::minutes(delay_minutes as i64)
}

fn create_retry_metadata(job_metadata: &JobMetadataEntity) -> JobMetadataEntity {
    JobMetadataEntity {
        job_id: job_metadata.job_id,
        status: JobMetadataStatus::Scheduled,
        failure: None,
        processed_at: None,
    }
}

fn create_failed_metadata(job_metadata: &JobMetadataEntity, error_msg: String) -> JobMetadataEntity {
    JobMetadataEntity {
        job_id: job_metadata.job_id,
        failure: Some(error_msg),
        processed_at: None,
        status: JobMetadataStatus::Failed,
    }
}

async fn handle_retry<JR>(
    job: &JobEntity,
    job_metadata: &JobMetadataEntity,
    job_repository: JR,
    config: &AppConfig,
) -> Result<()>
where
    JR: JobRepositoryPort + Send + Sync,
{
    let new_time = backoff(job.time, job.retries + 1, config.engine.base_delay_minutes);
    let retry_metadata = create_retry_metadata(job_metadata);

    info!("Retrying job {} (attempt {}/{})", job.id, job.retries + 1, config.engine.retry_attempts);

    job_repository
        .handle_retry_transaction(job.id, new_time, retry_metadata)
        .await
        .map_err(TempusError::Database)
        .map(|_| increment_jobs_processed("retry"))
}

async fn handle_permanent_failure<JMR>(
    job: &JobEntity,
    job_metadata: &JobMetadataEntity,
    job_metadata_repository: JMR,
    error_msg: String,
) -> Result<()>
where
    JMR: JobMetadataRepositoryPort + Send + Sync,
{
    warn!("Job {} failed permanently after {} attempts: {}", job.id, job.retries, error_msg);

    let failed_metadata = create_failed_metadata(job_metadata, error_msg);

    job_metadata_repository
        .update_status(failed_metadata)
        .await
        .map_err(TempusError::Database)
        .map(|_| increment_jobs_processed("failure"))
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
    match should_retry(job.retries, config.engine.retry_attempts) {
        true => handle_retry(&job, &job_metadata, job_repository, config).await,
        false => handle_permanent_failure(&job, &job_metadata, job_metadata_repository, error_msg).await,
    }
}

async fn process_http_job<JMR>(
    job: &JobEntity,
    metadata: JobMetadataEntity,
    target: String,
    payload: JsonValue,
    job_metadata_repository: JMR,
) -> Result<()>
where
    JMR: JobMetadataRepositoryPort + Send + Sync + 'static,
{
    perform_request(target, payload)
        .await
        .and_then(|response| {
            increment_http_requests(response.status().as_u16());
            info!("Job {} completed successfully", job.id);
            Ok(())
        })
        .map_err(|e| {
            error!("Job {} failed: {}", job.id, e);
            e
        })?;

    handle_success(metadata, job_metadata_repository).await
}

async fn process_kafka_job<JMR>(
    job: &JobEntity,
    metadata: JobMetadataEntity,
    target: String,
    payload: JsonValue,
    job_metadata_repository: JMR,
) -> Result<()>
where
    JMR: JobMetadataRepositoryPort + Send + Sync + 'static,
{
    publish_kafka_message(target, payload)
        .await
        .map_err(|e| {
            error!("Kafka job {} failed: {}", job.id, e);
            e
        })?;

    increment_kafka_messages();
    info!("Kafka job {} completed successfully", job.id);
    handle_success(metadata, job_metadata_repository).await
}

async fn process_job_with_metadata<JR, JMR>(
    job: &JobEntity,
    inner_job: &JobEntity,
    metadata: JobMetadataEntity,
    target: String,
    payload: JsonValue,
    job_repository: JR,
    job_metadata_repository: JMR,
    config: &AppConfig,
) -> Result<()>
where
    JR: JobRepositoryPort + Send + Sync,
    JMR: JobMetadataRepositoryPort + Send + Sync + Clone + 'static,
{
    let job_result = match job.r#type {
        JobType::Http => process_http_job(job, metadata.clone(), target, payload, job_metadata_repository.clone()).await,
        JobType::Kafka => process_kafka_job(job, metadata.clone(), target, payload, job_metadata_repository.clone()).await,
    };

    match job_result {
        Ok(_) => Ok(()),
        Err(e) => handle_failure(
            inner_job.clone(),
            metadata,
            job_repository,
            job_metadata_repository,
            e.to_string(),
            config,
        ).await,
    }
}

async fn process_job_by_type<JR, JMR>(
    job: &JobEntity,
    inner_job: &JobEntity,
    target: String,
    payload: JsonValue,
    job_repository: JR,
    job_metadata_repository: JMR,
    config: &AppConfig,
) -> Result<()>
where
    JR: JobRepositoryPort + Send + Sync,
    JMR: JobMetadataRepositoryPort + Send + Sync + Clone + 'static,
{
    match &job.metadata {
        None => {
            warn!("Metadata is missing for jobId: {}", &job.id);
            Err(TempusError::JobProcessing("Missing job metadata".to_string()))
        }
        Some(metadata) => {
            process_job_with_metadata(
                job,
                inner_job,
                metadata.clone(),
                target,
                payload,
                job_repository,
                job_metadata_repository,
                config,
            ).await
        }
    }
}

impl<JR, JMR> ProcessJobUseCasePort for ProcessJobUseCase<JR, JMR>
where
    JR: JobRepositoryPort + Send + Sync + Clone + 'static,
    JMR: JobMetadataRepositoryPort + Send + Sync + Clone + 'static,
{
    async fn execute(&self) -> Result<()> {
        let start_time = Instant::now();
        
        let jobs = self
            .job_repository
            .find_and_flag_processing(self.config.engine.max_concurrent_jobs)
            .await
            .map_err(TempusError::Database)?;

        if jobs.is_empty() {
            return Ok(());
        }

        let jobs_count = jobs.len();
        info!("Processing {} jobs", jobs_count);
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
                let job_start_time = Instant::now();
                increment_current_processing_jobs();
                
                let result = process_job_by_type(
                    &job,
                    &inner_job,
                    job_target,
                    job_payload,
                    job_repository,
                    job_metadata_repository,
                    &config,
                ).await;
                
                if let Err(e) = result {
                    error!("Error processing job {}: {:?}", job.id, e);
                }

                let duration = job_start_time.elapsed();
                observe_job_duration(duration.as_secs_f64());
                decrement_current_processing_jobs();
            });
            
            handles.push(handle);
        }

        for handle in handles {
            if let Err(e) = handle.await {
                error!("Task join error: {}", e);
            }
        }

        let total_duration = start_time.elapsed();
        info!("Completed processing {} jobs in {:.3}s", jobs_count, total_duration.as_secs_f64());

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