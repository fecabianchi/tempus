use crate::domain::job::entity::job_metadata_entity::JobMetadataEntity;
use crate::domain::job::port::driven::job_metadata_repository_port::JobMetadataRepositoryPort;
use crate::domain::job::port::driven::job_repository_port::JobRepositoryPort;
use crate::domain::job::port::driver::process_job_use_case_port::ProcessJobUseCasePort;
use crate::domain::job::r#enum::job_enum::{JobMetadataStatus, JobType};
use chrono::Utc;
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

impl<JR, JMR> ProcessJobUseCasePort for ProcessJobUseCase<JR, JMR>
where
    JR: JobRepositoryPort + Send + Sync,
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
            let job_metadata_repository = self.job_metadata_repository.clone();
            let job_target = job.target.clone();
            let job_payload = job.payload.clone();

            tokio::spawn(async move {
                let _permit = permit;
                match job.r#type {
                    JobType::Http => match job.metadata {
                        None => info!("Metadata is missing for jobId: {}", &job.id),
                        Some(metadata) => {
                            let result = perform_request(job_target, job_payload).await;

                            let to_update = match result {
                                Ok(_) => JobMetadataEntity {
                                    job_id: metadata.job_id,
                                    status: JobMetadataStatus::Completed,
                                    failure: None,
                                    processed_at: Some(Utc::now().naive_utc()),
                                },
                                Err(e) => JobMetadataEntity {
                                    job_id: metadata.job_id,
                                    status: JobMetadataStatus::Failed,
                                    failure: Some(e.to_string()),
                                    processed_at: Some(Utc::now().naive_utc()),
                                },
                            };

                            job_metadata_repository.update_status(to_update).await
                        }
                    },
                }
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
