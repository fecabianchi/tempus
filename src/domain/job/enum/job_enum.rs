#[derive(Debug, Clone)]
pub enum JobType {
    Http
}

#[derive(Debug, Clone)]
pub enum JobMetadataStatus {
    Scheduled,
    Completed,
    Deleted,
    Failed,
}
