use config::get_config;
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
    info!("get config");

    let mut fetcher = fetcher::Fetcher::new(config);
    fetcher.fetch().await?;
    info!("fetch done");

    Ok(())
}

fn log_init() {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::DEBUG)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
}
