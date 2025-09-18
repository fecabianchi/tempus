mod api;
mod config;
mod domain;
mod engine;
mod error;
mod infrastructure;

use crate::engine::TempusEngine;
use crate::engine::TempusEnginePort;
use crate::error::Result;
use crate::infrastructure::metrics::init_metrics;
use crate::infrastructure::metrics_server::start_metrics_server;
use log::info;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv()?;
    env_logger::init();
    
    match init_metrics() {
        Ok(_) => {
            info!("Metrics system initialized successfully");
            
            tokio::spawn(async {
                if let Err(e) = start_metrics_server(3001).await {
                    log::error!("Failed to start metrics server: {}", e);
                }
            });
        }
        Err(e) => {
            eprintln!("Failed to initialize metrics: {}", e);
        }
    }

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
