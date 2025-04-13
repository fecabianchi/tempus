use std::time::Duration;
use sea_orm::{ConnectOptions, Database, DatabaseConnection};

pub async fn connect() -> DatabaseConnection {
    let mut connection_options = ConnectOptions::new(dotenvy::var("DATABASE_URL").unwrap());
        connection_options.max_connections(100)
        .min_connections(5)
        .connect_timeout(Duration::from_secs(8))
        .acquire_timeout(Duration::from_secs(8))
        .idle_timeout(Duration::from_secs(8))
        .max_lifetime(Duration::from_secs(8))
        .sqlx_logging(true)
        .sqlx_logging_level(log::LevelFilter::Info);

    Database::connect(connection_options)
        .await
        .expect("Failed to connect to the database")
}