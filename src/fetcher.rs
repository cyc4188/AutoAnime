use anyhow::Result;
use reqwest::Client;
use rss::Channel;

use crate::config::Config;

pub struct Fetcher {
    config: Config,
    request_client: Client,
}

impl Fetcher {
    pub async fn fetch(&self) -> Result<()> {
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
                subscriber.notify(&self.config, channel.clone()).await?;
            }
        }
        Ok(())
    }
}
