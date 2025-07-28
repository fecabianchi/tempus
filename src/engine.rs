use crate::config::connection::connect_with_retry;
use crate::domain::job::port::driver::process_job_use_case_port::ProcessJobUseCasePort;
use crate::domain::job::usecase::process_job_use_case::ProcessJobUseCase;
use crate::infrastructure::persistence::job::job_metadata_repository::JobMetadataRepository;
use crate::infrastructure::persistence::job::job_repository::JobRepository;
use log::info;

pub trait TempusEnginePort {
    async fn start(&self) -> ();
}

pub struct TempusEngine;

impl TempusEnginePort for TempusEngine {
    async fn start(&self) -> () {
        let database = connect_with_retry().await;
        let job_repository = JobRepository::new(database.clone());
        let job_metadata_repository = JobMetadataRepository::new(database.clone());
        let usecase = ProcessJobUseCase::new(job_repository, job_metadata_repository);

        info!("Engine starting");

        loop {
            usecase.execute().await;
        }
    }
}
