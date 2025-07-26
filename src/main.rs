mod config;
mod domain;
mod engine;
mod infrastructure;

use crate::engine::TempusEngine;
use crate::engine::TempusEnginePort;
use log::info;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
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
    TempusEngine.start().await;

    Ok(())
}
