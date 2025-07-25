use crate::domain::job::entity::job_metadata_entity::JobMetadataEntity;
use crate::domain::job::r#enum::job_enum::{JobMetadataStatus, JobType};
use crate::domain::job::port::driven::job_metadata_repository_port::JobMetadataRepositoryPort;
use crate::domain::job::port::driven::job_repository_port::JobRepositoryPort;
use crate::domain::job::port::driver::process_job_use_case_port::ProcessJobUseCasePort;
use chrono::Utc;
use reqwest::{Client, Error, Response};
use sea_orm::JsonValue;

pub struct ProcessJobUseCase<
    JR: JobRepositoryPort + Send + Sync,
    JMR: JobMetadataRepositoryPort + Send + Sync,
> {
    job_repository: JR,
    job_metadata_repository: JMR,
}

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
            .find_all()
            .await
            .expect("TODO: panic message");

        for job in jobs {
            let job_metadata_repository = self.job_metadata_repository.clone();
            let job_target = job.target.clone();
            let job_payload = job.payload.clone();

            tokio::spawn(async move {
                match job.r#type {
                    JobType::Http => match job.metadata {
                        None => println!("Metadata is missing for jobId: {}", &job.id),
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
    Client::new()
        .post(target)
        .json(&serde_json::json!(payload))
        .send()
        .await
}
