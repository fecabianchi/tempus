use tempus::api::routes;
use tempus::config::app_config::AppConfig;
use tempus::config::connection::connect_with_retry;
use tempus::infrastructure::persistence::job::job_repository::JobRepository;
use tempus::error::Result;
use axum::serve;
use log::info;
use std::net::SocketAddr;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv()?;
    env_logger::init();

    let logo = r#"
▀▀█▀▀ █▀▀ █▀▄▀█ █▀▀█ █  █ █▀▀   █▀▀█ █▀▀█ ▀█▀
  █   █▀▀ █ ▀ █ █  █ █  █ ▀▀█   █▄▄█ █  █  █
  ▀   ▀▀▀ ▀   ▀ █▀▀▀ ▀▀▀▀ ▀▀▀   ▀  ▀ █▀▀▀ ▀▀▀
    "#;

    info!("{}", logo);
    info!("Starting Tempus API Server");

    let config = AppConfig::load()?;
    let database = connect_with_retry(&config).await?;
    let job_repository = JobRepository::new(database);

    let app = routes::create_router(job_repository);
    let addr = SocketAddr::from(([0, 0, 0, 0], config.http.port));
    let listener = TcpListener::bind(addr).await?;
    
    info!("Tempus API listening on {}", addr);

    serve(listener, app).await?;

    Ok(())
}
