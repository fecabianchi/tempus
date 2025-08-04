use crate::config::app_config::AppConfig;
use crate::error::{Result, TempusError};
use log::{error, info};
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use std::time::Duration;
use tokio::time::sleep;

async fn connect(config: &AppConfig) -> Result<DatabaseConnection> {
    let database_url = std::env::var("DATABASE_URL")
        .or_else(|_| dotenvy::var("DATABASE_URL"))
        .map_err(|_| TempusError::Config("DATABASE_URL environment variable not set".to_string()))?;

    let mut connection_options = ConnectOptions::new(database_url);
    connection_options
        .max_connections(config.database.max_connections)
        .min_connections(config.database.min_connections)
        .connect_timeout(config.database.connect_timeout())
        .acquire_timeout(config.database.acquire_timeout())
        .idle_timeout(config.database.idle_timeout())
        .max_lifetime(config.database.max_lifetime())
        .sqlx_logging(true)
        .sqlx_logging_level(log::LevelFilter::Debug);

    Database::connect(connection_options)
        .await
        .map_err(TempusError::Database)
}

pub async fn connect_with_retry(config: &AppConfig) -> Result<DatabaseConnection> {
    let mut retry_count = 0;
    const MAX_RETRIES: u32 = 10;
    
    loop {
        match connect(config).await {
            Ok(conn) => {
                info!("Successfully connected to the database");
                return Ok(conn);
            }
            Err(e) => {
                retry_count += 1;
                if retry_count >= MAX_RETRIES {
                    error!("Failed to connect to database after {} attempts", MAX_RETRIES);
                    return Err(e);
                }
                
                error!("Failed to connect to DB (attempt {}/{}): {:?}. Retrying in 5s...", 
                       retry_count, MAX_RETRIES, e);
                sleep(Duration::from_secs(5)).await;
            }
        }
    }
}

pub fn is_connection_error(err: &sea_orm::DbErr) -> bool {
    matches!(err, sea_orm::DbErr::Conn(_) | sea_orm::DbErr::ConnectionAcquire(_))
}
