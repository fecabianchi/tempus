use crate::config::connection::{connect_with_retry, is_connection_error};
use crate::usecase::process_scheduled_jobs::process;
use std::time::Duration;
use tokio::time::sleep;

pub async fn start() {
    println!("TEMPUS ENGINE: Starting");
    let mut db = connect_with_retry().await;

    let interval = dotenvy::var("POLL_INTERVAL")
        .unwrap_or_else(|_| "10".into())
        .parse::<u64>()
        .unwrap_or(10);

    loop {
        println!("TEMPUS ENGINE: Fetching scheduled events");
        match process(&db).await {
            Ok(_) => println!("TEMPUS ENGINE: Events processed"),
            Err(err) => {
                eprintln!("TEMPUS ENGINE:️ Error processing jobs: {:?}", err);

                if is_connection_error(&err) {
                    println!("Reconnecting to DB...");
                    db = connect_with_retry().await;
                }
            }
        }

        sleep(Duration::from_secs(interval)).await;
    }
}

