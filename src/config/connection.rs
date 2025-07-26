use log::{error, info};
use sea_orm::{ConnectOptions, Database, DatabaseConnection, DbErr};
use std::time::Duration;
use tokio::time::sleep;

async fn connect() -> Result<DatabaseConnection, DbErr> {
    let mut connection_options = ConnectOptions::new(dotenvy::var("DATABASE_URL").unwrap());
    connection_options
        .max_connections(100)
        .min_connections(5)
        .connect_timeout(Duration::from_secs(8))
        .acquire_timeout(Duration::from_secs(8))
        .idle_timeout(Duration::from_secs(8))
        .max_lifetime(Duration::from_secs(8))
        .sqlx_logging(true)
        .sqlx_logging_level(log::LevelFilter::Debug);

    Database::connect(connection_options).await
}

pub async fn connect_with_retry() -> DatabaseConnection {
    loop {
        match connect().await {
            Ok(conn) => {
                info!("Connected to the database");
                return conn;
            }
            Err(e) => {
                error!("Failed to connect to DB: {e:?}. Retrying in 5s...");
                sleep(Duration::from_secs(5)).await
            }
        }
    }
}

pub fn is_connection_error(err: &DbErr) -> bool {
    matches!(err, DbErr::Conn(_) | DbErr::ConnectionAcquire(_))
}
