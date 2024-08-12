use std::sync::Arc;

use auto_anime::{
    config::{get_config, Config},
    AutoAnime,
};
use delay_timer::{
    entity::DelayTimerBuilder,
    error::TaskError,
    prelude::{Task, TaskBuilder},
};
use tokio::sync::Mutex;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    log_init();

    let path = "./config.yaml";
    let config = Arc::new(get_config(path)?);
    info!("[Config] get config");

    let delay_timer = DelayTimerBuilder::default()
        .tokio_runtime_by_default()
        .build();

    delay_timer.add_task(fetch_task(config).await?)?;
    info!("[Task] add task");

    loop {
        tokio::time::sleep(std::time::Duration::from_secs(30)).await;
    }

    Ok(())
}

async fn fetch_task(config: Arc<Config>) -> anyhow::Result<Task, TaskError> {
    let mut task_builder = TaskBuilder::default();
    let auto_anime = Arc::new(Mutex::new(
        AutoAnime::new(config).expect("[AutoAnime] Create Failed"),
    ));

    {
        if let Err(e) = auto_anime.lock().await.run().await {
            info!("[Task] Run Error: {}", e);
        }
        info!("[Task] Fetch Done");
    }

    let body = move || {
        let auto_anime = auto_anime.clone();
        async move {
            if let Err(e) = auto_anime.lock().await.run().await {
                info!("[Task] Run Error: {}", e);
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
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
}
