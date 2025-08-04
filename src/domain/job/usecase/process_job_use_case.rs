use crate::domain::job::entity::job_entity::JobEntity;
use crate::domain::job::entity::job_metadata_entity::JobMetadataEntity;
use crate::domain::job::r#enum::job_enum::{JobMetadataStatus, JobType};
use crate::domain::job::port::driven::job_metadata_repository_port::JobMetadataRepositoryPort;
use crate::domain::job::port::driven::job_repository_port::JobRepositoryPort;
use crate::domain::job::port::driver::process_job_use_case_port::ProcessJobUseCasePort;
use chrono::{NaiveDateTime, Utc};
use log::info;
use once_cell::sync::Lazy;
use reqwest::{Client, Error, Response};
use sea_orm::JsonValue;
use std::sync::Arc;

pub struct ProcessJobUseCase<
    JR: JobRepositoryPort + Send + Sync,
    JMR: JobMetadataRepositoryPort + Send + Sync,
> {
    job_repository: JR,
    job_metadata_repository: JMR,
}

static HTTP_CLIENT: Lazy<Client> = Lazy::new(|| {
    Client::builder()
        .pool_idle_timeout(std::time::Duration::from_secs(30))
        .build()
        .unwrap()
});

impl<JR: JobRepositoryPort + Send + Sync, JMR: JobMetadataRepositoryPort + Send + Sync>
    ProcessJobUseCase<JR, JMR>
{
    pub fn new(job_repository: JR, job_metadata_repository: JMR) -> Self {
        Self {
            job_repository,
            job_metadata_repository,
        }
    }
}

async fn handle_success<JMR>(metadata: JobMetadataEntity, job_metadata_repository: JMR)
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
        .await;
}

fn should_retry(retries: i32) -> bool {
    retries < 3
}

fn backoff(time: NaiveDateTime, retries: i32) -> NaiveDateTime {
    let delay_minutes = 2u32.pow(retries as u32);
    time + chrono::Duration::minutes(delay_minutes as i64)
}

async fn handle_failure<JR, JMR>(
    job: JobEntity,
    job_metadata: JobMetadataEntity,
    job_repository: JR,
    job_metadata_repository: JMR,
    error_msg: String,
) where
    JR: JobRepositoryPort + Send + Sync,
    JMR: JobMetadataRepositoryPort + Send + Sync,
{
    let current_retries = job.retries;
    if should_retry(current_retries) {
        job_repository
            .increment_retry(job.id)
            .await
            .expect("TODO: panic message");
        job_repository
            .update_time(job.id, backoff(job.time, current_retries + 1))
            .await
            .expect("");
        
        let retry_metadata = JobMetadataEntity {
            job_id: job_metadata.job_id,
            status: JobMetadataStatus::Scheduled,
            failure: None,
            processed_at: None,
        };
        job_metadata_repository.update_status(retry_metadata).await;
    } else {
        let failed_metadata = JobMetadataEntity {
            job_id: job_metadata.job_id,
            failure: Some(error_msg),
            processed_at: None,
            status: JobMetadataStatus::Failed,
        };

        job_metadata_repository.update_status(failed_metadata).await
    }
}

impl<JR, JMR> ProcessJobUseCasePort for ProcessJobUseCase<JR, JMR>
where
    JR: JobRepositoryPort + Send + Sync + Clone + 'static,
    JMR: JobMetadataRepositoryPort + Send + Sync + Clone + 'static,
{
    async fn execute(&self) {
        let jobs = self
            .job_repository
            .find_and_flag_processing()
            .await
            .expect("TODO: panic message");

        let semaphore = Arc::new(tokio::sync::Semaphore::new(10));

        for job in jobs {
            let permit = semaphore.clone().acquire_owned().await.unwrap();
            let job_repository = self.job_repository.clone();
            let job_metadata_repository = self.job_metadata_repository.clone();
            let job_target = job.target.clone();
            let job_payload = job.payload.clone();
            let inner_job = job.clone();

            tokio::spawn(async move {
                let _permit = permit;
                match job.r#type {
                    JobType::Http => match job.metadata {
                        None => info!("Metadata is missing for jobId: {}", &job.id),
                        Some(metadata) => {
                            match perform_request(job_target, job_payload).await {
                                Ok(_) => {
                                    handle_success(metadata, job_metadata_repository).await;
                                }
                                Err(e) => {
                                    handle_failure(
                                        inner_job,
                                        metadata,
                                        job_repository,
                                        job_metadata_repository,
                                        e.to_string(),
                                    )
                                    .await;
                                }
                            };
                        }
                    },
                };
            });
        }
    }
}

async fn perform_request(target: String, payload: JsonValue) -> Result<Response, Error> {
    HTTP_CLIENT
        .post(target)
        .json(&serde_json::json!(payload))
        .send()
        .await
}
