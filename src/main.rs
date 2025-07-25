mod config;
mod domain;
mod engine;
mod infrastructure;

use crate::config::connection::connect_with_retry;
use crate::domain::job::usecase::process_job_use_case::ProcessJobUseCase;
use crate::engine::TempusEngine;
use crate::engine::TempusEnginePort;
use crate::infrastructure::persistence::job::job_repository::JobRepository;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenvy::dotenv()?;

    let database = connect_with_retry().await;
    let job_repository = JobRepository::new(database.clone());
    let usecase = ProcessJobUseCase::new(job_repository);
    let engine = TempusEngine::new(usecase);

    engine.start().await;

    Ok(())
}
