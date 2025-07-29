mod config;
mod domain;
mod engine;
mod infrastructure;

use crate::engine::TempusEngine;
use crate::engine::TempusEnginePort;
use log::info;
use std::error::Error;
use tokio::signal;

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

    let engine = TempusEngine::new();

    tokio::select! {
        _ = engine.start() => {
            info!("Engine stopped");
        }
        _ = signal::ctrl_c() => {
            info!("Received SIGINT (Ctrl+C), initiating graceful shutdown...");
            engine.shutdown().await;
        }
        _ = async {
            use tokio::signal::unix::{signal, SignalKind};
            let mut term = signal(SignalKind::terminate()).expect("Failed to register SIGTERM handler");
            term.recv().await;
        } => {
            info!("Received SIGTERM, initiating graceful shutdown...");
            engine.shutdown().await;
        }
    }

    Ok(())
}
