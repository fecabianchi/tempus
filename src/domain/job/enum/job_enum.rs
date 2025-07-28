#[derive(Debug, Clone)]
pub enum JobType {
    Http
}

#[derive(Debug, Clone)]
pub enum JobMetadataStatus {
    Scheduled,
    Processing,
    Completed,
    Deleted,
    Failed,
}
