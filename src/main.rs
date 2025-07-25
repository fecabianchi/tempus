mod config;
mod domain;
mod engine;
mod infrastructure;

use crate::engine::TempusEngine;
use crate::engine::TempusEnginePort;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenvy::dotenv()?;

    TempusEngine.start().await;

    Ok(())
}
