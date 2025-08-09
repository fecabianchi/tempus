use log::info;
use uuid::Uuid;

use crate::domain::job::port::driven::job_repository_port::JobRepositoryPort;
use crate::error::{TempusError, Result};

pub struct DeleteJobUseCase<R: JobRepositoryPort> {
    job_repository: R,
}

impl<R: JobRepositoryPort> DeleteJobUseCase<R> {
    pub fn new(job_repository: R) -> Self {
        Self { job_repository }
    }

    pub async fn execute(&self, job_id: Uuid) -> Result<()> {
        let job_deleted = self.job_repository.delete_unprocessed(job_id).await
            .map_err(TempusError::from)?;

        if !job_deleted {
            return Err(TempusError::Validation(
                "Job not found or already processed".to_string()
            ));
        }

        info!("Job deleted successfully with ID: {}", job_id);
        Ok(())
    }
}