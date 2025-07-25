use crate::domain::job::port::driven::job_repository_port::JobRepositoryPort;
use crate::domain::job::port::driver::process_job_use_case_port::ProcessJobUseCasePort;
use crate::domain::job::usecase::process_job_use_case::ProcessJobUseCase;

pub trait TempusEnginePort {
    async fn start(&self) -> ();
}

pub struct TempusEngine<JR: JobRepositoryPort + Send + Sync> {
    process_job_use_case: ProcessJobUseCase<JR>,
}

impl<JR: JobRepositoryPort + Send + Sync> TempusEngine<JR> {
    pub fn new(process_job_use_case: ProcessJobUseCase<JR>) -> Self {
        Self {
            process_job_use_case,
        }
    }
}

impl<JR: JobRepositoryPort + Send + Sync> TempusEnginePort for TempusEngine<JR> {
    async fn start(&self) -> () {
        println!("TEMPUS ENGINE: Starting");
        self.process_job_use_case.execute().await;
    }
}

// use crate::config::connection::{connect_with_retry, is_connection_error};
// use crate::usecase::process_scheduled_jobs::process;
// use std::time::Duration;
// use tokio::time::sleep;
//
// pub async fn start() {
//     println!("TEMPUS ENGINE: Starting");
//     let mut db = connect_with_retry().await;
//
//     let interval = dotenvy::var("POLL_INTERVAL")
//         .unwrap_or_else(|_| "10".into())
//         .parse::<u64>()
//         .unwrap_or(10);
//
//     loop {
//         println!("TEMPUS ENGINE: Fetching scheduled events");
//         match process(&db).await {
//             Ok(_) => println!("TEMPUS ENGINE: Events processed"),
//             Err(err) => {
//                 eprintln!("TEMPUS ENGINE:️ Error processing jobs: {:?}", err);
//
//                 if is_connection_error(&err) {
//                     println!("Reconnecting to DB...");
//                     db = connect_with_retry().await;
//                 }
//             }
//         }
//
//         sleep(Duration::from_secs(interval)).await;
//     }
// }
//
