use crate::domain::job::r#enum::job_enum::JobType;
use crate::domain::job::port::driven::job_repository_port::JobRepositoryPort;
use crate::domain::job::port::driver::process_job_use_case_port::ProcessJobUseCasePort;

pub struct ProcessJobUseCase<JR: JobRepositoryPort + Send + Sync> {
    job_repository: JR,
}

impl<JR: JobRepositoryPort + Send + Sync> ProcessJobUseCase<JR> {
    pub fn new(job_repository: JR) -> Self {
        Self { job_repository }
    }
}

impl<JR: JobRepositoryPort + Send + Sync> ProcessJobUseCasePort for ProcessJobUseCase<JR> {
    async fn execute(&self) -> () {
        let jobs = self
            .job_repository
            .find_all()
            .await
            // Add proper error handling
            .expect("TODO: panic message");

        for job in jobs {
            match job.r#type {
                JobType::Http => {
                    println!("Processing jobId: {}", job.id)
                }
            }
        }
    }
}
