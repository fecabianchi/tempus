use chrono::NaiveDateTime;
use log::info;
use uuid::Uuid;

use crate::domain::job::port::driven::job_repository_port::JobRepositoryPort;
use crate::error::{TempusError, Result};

pub struct UpdateJobTimeUseCase<R: JobRepositoryPort> {
    job_repository: R,
}

impl<R: JobRepositoryPort> UpdateJobTimeUseCase<R> {
    pub fn new(job_repository: R) -> Self {
        Self { job_repository }
    }

    pub async fn execute(&self, job_id: Uuid, new_time: NaiveDateTime) -> Result<()> {
        let job_updated = self.job_repository.update_time_unprocessed(job_id, new_time).await
            .map_err(TempusError::from)?;

        if !job_updated {
            return Err(TempusError::Validation(
                "Job not found or already processed".to_string()
            ));
        }

        info!("Job time updated successfully for ID: {} to: {}", job_id, new_time);
        Ok(())
    }
}