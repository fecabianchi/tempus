use thiserror::Error;

#[derive(Error, Debug)]
pub enum TempusError {
    #[error("Database error: {0}")]
    Database(#[from] sea_orm::DbErr),
    
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),
    
    #[error("Configuration error: {0}")]
    Config(String),
    
    #[error("Environment variable error: {0}")]
    Environment(#[from] dotenvy::Error),
    
    #[error("Job processing error: {0}")]
    JobProcessing(String),
    
    #[error("Validation error: {0}")]
    Validation(String),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Kafka error: {0}")]
    Kafka(String),
}

pub type Result<T> = std::result::Result<T, TempusError>;