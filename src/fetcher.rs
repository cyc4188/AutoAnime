use anyhow::Result;
use chrono::{DateTime, Local};
use reqwest::Client;
use rss_for_mikan::Channel;

use crate::config::Config;

pub struct Fetcher {
    config: Config,
    request_client: Client,
    last_update_time: DateTime<Local>,
}

impl Fetcher {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            request_client: Client::new(),
            last_update_time: Local::now(),
        }
    }

    /// fetch feeds
    pub async fn fetch(&mut self) -> Result<()> {
        // TODO add filter
        for feed in self.config.feeds() {
            let content = self
                .request_client
                .get(feed.url.clone())
                .send()
                .await?
                .bytes()
                .await?;
            let channel = Channel::read_from(&content[..])?;

            for subscriber in &feed.subscriber {
                subscriber.notify(&self.config, &channel).await?;
            }
        }
        self.last_update_time = Local::now();
        Ok(())
    }
}
