use axum::http::StatusCode;

pub async fn health_check() -> Result<&'static str, StatusCode> {
    Ok("OK")
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_health_check() {
        let result = health_check().await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "OK");
    }
}