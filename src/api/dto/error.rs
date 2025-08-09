use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ApiError {
    pub error: String,
    pub message: String,
}

impl ApiError {
    pub fn new(error: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            error: error.into(),
            message: message.into(),
        }
    }
    
    pub fn validation_error(message: impl Into<String>) -> Self {
        Self::new("validation_failed", message)
    }
    
    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::new("bad_request", message)
    }
    
    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new("not_found", message)
    }
    
    pub fn internal_error(message: impl Into<String>) -> Self {
        Self::new("internal_error", message)
    }
}