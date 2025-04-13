mod config;
mod engine;
mod usecase;
mod entity;

use std::error::Error;
use dotenvy;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenvy::dotenv()?;
    engine::start().await;
    Ok(())
}
