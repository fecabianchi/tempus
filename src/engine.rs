use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use log::info;
use sea_orm::prelude::async_trait::async_trait;
use crate::config::connection::connect_with_retry;
use crate::domain::job::port::driver::process_job_use_case_port::ProcessJobUseCasePort;
use crate::domain::job::usecase::process_job_use_case::ProcessJobUseCase;
use crate::infrastructure::persistence::job::job_metadata_repository::JobMetadataRepository;
use crate::infrastructure::persistence::job::job_repository::JobRepository;

#[async_trait]
pub trait TempusEnginePort {
    async fn start(&self);
    async fn shutdown(&self);
}

pub struct TempusEngine {
    shutdown_signal: Arc<AtomicBool>,
}

impl TempusEngine {
    pub fn new() -> Self {
        Self {
            shutdown_signal: Arc::new(AtomicBool::new(false)),
        }
    }
}

#[async_trait]
impl TempusEnginePort for TempusEngine {
    async fn start(&self) {
        let database = connect_with_retry().await;
        let job_repository = JobRepository::new(database.clone());
        let job_metadata_repository = JobMetadataRepository::new(database);
        let usecase = ProcessJobUseCase::new(job_repository, job_metadata_repository);

        info!("Engine starting");

        loop {
            if self.shutdown_signal.load(Ordering::Relaxed) {
                info!("Shutdown signal received, exiting the loop");
                break;
            }

            usecase.execute().await;
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
    }

    async fn shutdown(&self) {
        info!("Initiating graceful shutdown...");
        self.shutdown_signal.store(true, Ordering::Relaxed);
        info!("Shutdown signal set");
    }
}

