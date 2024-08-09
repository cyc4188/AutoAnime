use std::sync::Arc;

use config::{get_config, Config};
use delay_timer::{
    entity::DelayTimerBuilder,
    error::TaskError,
    prelude::{Task, TaskBuilder},
};
use tokio::sync::Mutex;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

mod config;
mod feeds;
mod fetcher;
mod subscriber;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    log_init();

    let path = "./config.yaml";
    let config = get_config(path)?;
    info!("[Config] get config");

    let delay_timer = DelayTimerBuilder::default()
        .tokio_runtime_by_default()
        .build();

    delay_timer.add_task(fetch_task(config)?)?;
    info!("[Task] add task");

    loop {
        tokio::time::sleep(std::time::Duration::from_secs(30)).await;
    }

    Ok(())
}

fn fetch_task(config: Config) -> anyhow::Result<Task, TaskError> {
    let mut task_builder = TaskBuilder::default();
    let fetcher = Arc::new(Mutex::new(fetcher::Fetcher::new(config)));

    let body = move || {
        let fetcher = fetcher.clone();
        async move {
            if let Err(e) = fetcher.lock().await.fetch().await {
                info!("[Task] Fetch Error: {}", e);
            }
            info!("[Task] Fetch Done");
        }
    };

    task_builder
        .set_task_id(1)
        .set_frequency_repeated_by_hours(6)
        .spawn_async_routine(body)
}

fn log_init() {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::DEBUG)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
}
