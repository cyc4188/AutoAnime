use std::sync::Arc;

use config::Config;
use distributor::Distributor;
use fetcher::Fetcher;
use tracing::error;

pub mod config;
mod distributor;
mod feeds;
mod fetcher;
mod subscriber;

pub struct AutoAnime {
    fetcher: Fetcher,
    pub distubtor: Distributor,
    _config: Arc<Config>,
}

impl AutoAnime {
    pub fn new(config: Arc<Config>) -> anyhow::Result<Self> {
        let fetcher = Fetcher::new(config.clone())?;
        let distubtor = Distributor::new(config.clone());
        Ok(Self {
            fetcher,
            distubtor,
            _config: config,
        })
    }
    pub async fn run(&mut self) -> anyhow::Result<()> {
        // TODO: make it concurrent
        let vec = self.fetcher.fetch().await?;
        for (channel, feed) in vec {
            for sub in &feed.subscriber {
                match self.distubtor.notify(&channel, sub).await {
                    Ok(_) => {}
                    Err(e) => {
                        error!("{}", e);
                    }
                }
            }
        }

        Ok(())
    }
}
