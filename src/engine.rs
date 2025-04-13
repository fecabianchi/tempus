use crate::config::connection::connect;
use std::time::Duration;
use tokio::time::sleep;
use crate::usecase::process_scheduled_jobs::process;

pub async fn start() {
    println!("TEMPUS ENGINE: Starting");
    let db = connect().await;

    loop {
        println!("TEMPUS ENGINE: Fetching scheduled events");
        process(&db).await.expect("TODO: handle processing errors");
        println!("TEMPUS ENGINE: Events processed");
        sleep(Duration::from_secs(10)).await;
    }
}
