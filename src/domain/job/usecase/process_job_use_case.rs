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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::job::entity::job_entity::JobEntity;
    use crate::domain::job::entity::job_metadata_entity::JobMetadataEntity;
    use crate::domain::job::r#enum::job_enum::{JobMetadataStatus, JobType};
    use chrono::DateTime;
    use mockall::mock;
    use sea_orm::prelude::async_trait::async_trait;
    use sea_orm::DbErr;
    use serde_json::json;
    use uuid::Uuid;

    mock! {
        JobRepo {}

        #[async_trait]
        impl JobRepositoryPort for JobRepo {
            async fn find_all(&self) -> Result<Vec<JobEntity>, DbErr>;
            async fn find_and_flag_processing(&self) -> Result<Vec<JobEntity>, DbErr>;
        }
    }

    mock! {
        JobMetadataRepo {}

        #[async_trait]
        impl JobMetadataRepositoryPort for JobMetadataRepo {
            async fn update_status(&self, job_metadata: JobMetadataEntity) -> ();
        }

        impl Clone for JobMetadataRepo {
            fn clone(&self) -> Self;
        }
    }

    #[test]
    fn test_process_job_use_case_new() {
        let job_repo = MockJobRepo::new();
        let metadata_repo = MockJobMetadataRepo::new();
        
        let _use_case = ProcessJobUseCase::new(job_repo, metadata_repo);
    }

    #[tokio::test]
    async fn test_execute_with_no_jobs() {
        let mut job_repo = MockJobRepo::new();
        let metadata_repo = MockJobMetadataRepo::new();
        
        job_repo
            .expect_find_and_flag_processing()
            .times(1)
            .returning(|| Ok(vec![]));
        
        let use_case = ProcessJobUseCase::new(job_repo, metadata_repo);
        
        use_case.execute().await;
    }

    #[tokio::test]
    async fn test_execute_with_job_without_metadata() {
        let mut job_repo = MockJobRepo::new();
        let mut metadata_repo = MockJobMetadataRepo::new();
        
        let job = JobEntity {
            id: Uuid::new_v4(),
            time: DateTime::from_timestamp(1000000000, 0).unwrap().naive_utc(),
            target: "http://example.com".to_string(),
            r#type: JobType::Http,
            payload: json!({"test": "data"}),
            metadata: None,
        };
        
        job_repo
            .expect_find_and_flag_processing()
            .times(1)    
            .returning(move || Ok(vec![job.clone()]));
        
        metadata_repo
            .expect_clone()
            .returning(MockJobMetadataRepo::new);
        
        let use_case = ProcessJobUseCase::new(job_repo, metadata_repo);
        
        use_case.execute().await;
        
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }

    #[tokio::test]
    async fn test_execute_with_job_with_metadata() {
        let mut job_repo = MockJobRepo::new();
        let mut metadata_repo = MockJobMetadataRepo::new();
        
        let job_id = Uuid::new_v4();
        let metadata = JobMetadataEntity {
            job_id,
            status: JobMetadataStatus::Processing,
            failure: None,
            processed_at: None,
        };
        
        let job = JobEntity {
            id: job_id,
            time: DateTime::from_timestamp(1000000000, 0).unwrap().naive_utc(),
            target: "http://invalid-url-for-testing".to_string(),
            r#type: JobType::Http,
            payload: json!({"test": "data"}),
            metadata: Some(metadata),
        };
        
        job_repo
            .expect_find_and_flag_processing()
            .times(1)
            .returning(move || Ok(vec![job.clone()]));
        
        metadata_repo
            .expect_clone()
            .returning(|| {
                let mut cloned_repo = MockJobMetadataRepo::new();
                cloned_repo
                    .expect_update_status()
                    .times(1)
                    .returning(|_| ());
                cloned_repo
            });
        
        let use_case = ProcessJobUseCase::new(job_repo, metadata_repo);
        
        use_case.execute().await;
        
        tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
    }
}
