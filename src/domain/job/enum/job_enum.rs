#[derive(Debug, Clone)]
pub enum JobType {
    Http,
    Kafka
}

#[derive(Debug, Clone)]
pub enum JobMetadataStatus {
    Scheduled,
    Processing,
    Completed,
    Deleted,
    Failed,
}
