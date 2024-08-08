use anyhow::Result;
use chrono::{DateTime, Days, Local};
use reqwest::Client;
use rss_for_mikan::Channel;
use tracing::info;

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
            last_update_time: Local::now().checked_sub_days(Days::new(1)).unwrap(),
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
            info!("Get Channel");
            let mut channel = Channel::read_from(&content[..])?;
            // filter channel
            if let Some(pub_date) = channel.pub_date() {
                let mut pub_date = pub_date.to_string();
                pub_date.push_str("+08:00");
                let pub_date = DateTime::parse_from_rfc3339(pub_date.as_str())
                    .unwrap()
                    .with_timezone(&Local {});
                if pub_date < self.last_update_time {
                    continue;
                }
            }

            // filter items
            channel.items.retain(|item| {
                if let Some(pub_date) = item.pub_date() {
                    let mut pub_date = pub_date.to_string();
                    pub_date.push_str("+08:00");
                    let pub_date = DateTime::parse_from_rfc3339(pub_date.as_str())
                        .unwrap()
                        .with_timezone(&Local {});
                    return pub_date > self.last_update_time;
                }
                true
            });

            for subscriber in &feed.subscriber {
                subscriber.notify(&self.config, &channel).await?;
            }
        }
        self.last_update_time = Local::now();
        Ok(())
    }
}
