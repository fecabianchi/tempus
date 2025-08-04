use crate::config::app_config::AppConfig;
use crate::config::connection::connect_with_retry;
use crate::domain::job::port::driver::process_job_use_case_port::ProcessJobUseCasePort;
use crate::domain::job::usecase::process_job_use_case::ProcessJobUseCase;
use crate::error::Result;
use crate::infrastructure::persistence::job::job_metadata_repository::JobMetadataRepository;
use crate::infrastructure::persistence::job::job_repository::JobRepository;
use log::{error, info, warn};
use std::time::Duration;
use tokio::signal;
use tokio::time::sleep;
use tokio_util::sync::CancellationToken;

pub trait TempusEnginePort {
    async fn start(&self) -> Result<()>;
}

pub struct TempusEngine {
    config: AppConfig,
}

impl TempusEngine {
    pub fn new() -> Result<Self> {
        let config = AppConfig::load()?;
        Ok(Self { config })
    }
}

impl TempusEnginePort for TempusEngine {
    async fn start(&self) -> Result<()> {
        info!("Starting Tempus Engine with graceful shutdown support");
        
        let database = connect_with_retry(&self.config).await?;
        let job_repository = JobRepository::new(database.clone());
        let job_metadata_repository = JobMetadataRepository::new(database.clone());
        let usecase = ProcessJobUseCase::new(job_repository, job_metadata_repository, &self.config);

        let shutdown_token = CancellationToken::new();
        let shutdown_token_clone = shutdown_token.clone();

        tokio::spawn(async move {
            match signal::ctrl_c().await {
                Ok(()) => {
                    info!("Received shutdown signal, initiating graceful shutdown...");
                    shutdown_token_clone.cancel();
                }
                Err(err) => {
                    error!("Failed to listen for shutdown signal: {}", err);
                }
            }
        });

        info!("Engine started, processing jobs...");

        while !shutdown_token.is_cancelled() {
            tokio::select! {
                result = usecase.execute() => {
                    if let Err(e) = result {
                        error!("Error processing jobs: {:?}", e);
                        sleep(Duration::from_secs(5)).await;
                    }
                }
                _ = shutdown_token.cancelled() => {
                    warn!("Shutdown signal received, stopping job processing");
                    break;
                }
                _ = sleep(Duration::from_millis(100)) => {
                }
            }
        }

        info!("Tempus Engine shutdown complete");
        Ok(())
    }
}
