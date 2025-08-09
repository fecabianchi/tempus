mod api;
mod config;
mod domain;
mod engine;
mod error;
mod infrastructure;

use crate::engine::TempusEngine;
use crate::engine::TempusEnginePort;
use crate::error::Result;
use log::info;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv()?;
    env_logger::init();

    let logo = r#"
                  ___           ___           ___         ___           ___
      ___        /  /\         /__/\         /  /\       /__/\         /  /\
     /  /\      /  /:/_       |  |::\       /  /::\      \  \:\       /  /:/_
    /  /:/     /  /:/ /\      |  |:|:\     /  /:/\:\      \  \:\     /  /:/ /\
   /  /:/     /  /:/ /:/_   __|__|:|\:\   /  /:/~/:/  ___  \  \:\   /  /:/ /::\
  /  /::\    /__/:/ /:/ /\ /__/::::| \:\ /__/:/ /:/  /__/\  \__\:\ /__/:/ /:/\:\
 /__/:/\:\   \  \:\/:/ /:/ \  \:\~~\__\/ \  \:\/:/   \  \:\ /  /:/ \  \:\/:/~/:/
 \__\/  \:\   \  \::/ /:/   \  \:\        \  \::/     \  \:\  /:/   \  \::/ /:/
      \  \:\   \  \:\/:/     \  \:\        \  \:\      \  \:\/:/     \__\/ /:/
       \__\/    \  \::/       \  \:\        \  \:\      \  \::/        /__/:/
                 \__\/         \__\/         \__\/       \__\/         \__\/
    "#;

    info!("{}", logo.to_ascii_lowercase());
    TempusEngine::new()?.start().await?;

    Ok(())
}
