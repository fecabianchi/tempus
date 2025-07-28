use crate::config::connection::connect_with_retry;
use crate::domain::job::port::driver::process_job_use_case_port::ProcessJobUseCasePort;
use crate::domain::job::usecase::process_job_use_case::ProcessJobUseCase;
use crate::infrastructure::persistence::job::job_metadata_repository::JobMetadataRepository;
use crate::infrastructure::persistence::job::job_repository::JobRepository;
use log::info;
use std::sync::Arc;
use tokio::sync::RwLock;

pub trait TempusEnginePort {
    async fn start(&self) -> ();
    async fn shutdown(&self);
}

pub struct TempusEngine {
    shutdown_signal: Arc<RwLock<bool>>,
}

impl TempusEngine {
    pub fn new() -> Self {
        Self {
            shutdown_signal: Arc::new(RwLock::new(false)),
        }
    }
}

impl TempusEnginePort for TempusEngine {
    async fn start(&self) -> () {
        let database = connect_with_retry().await;
        let job_repository = JobRepository::new(database.clone());
        let job_metadata_repository = JobMetadataRepository::new(database.clone());
        let usecase = ProcessJobUseCase::new(job_repository, job_metadata_repository);

        info!("Engine starting");

        loop {
            let should_shutdown = *self.shutdown_signal.read().await;
            if should_shutdown {
                info!("Shutdown signal received, exiting main loop");
                break;
            }
            
            usecase.execute().await;
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
    }

    async fn shutdown(&self) {
        info!("Initiating graceful shutdown...");
        let mut shutdown_signal = self.shutdown_signal.write().await;
        *shutdown_signal = true;
        info!("Shutdown signal set");
    }
}
