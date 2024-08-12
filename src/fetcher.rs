use std::{sync::Arc, time::Duration};

use anyhow::Result;
use chrono::{DateTime, Days, Local};
use reqwest::{Client, ClientBuilder, Proxy};
use rss_for_mikan::Channel;
use tracing::{debug, info};

use crate::{config::Config, feeds::Feed};

pub struct Fetcher {
    config: Arc<Config>,
    request_client: Client,
    last_update_time: DateTime<Local>,
}

impl Fetcher {
    pub fn new(config: Arc<Config>) -> anyhow::Result<Self> {
        let mut client_builder = Client::builder().connect_timeout(Duration::from_secs(2));
        if let Some(proxy) = config.proxy() {
            if let Ok(http_proxy) = Proxy::http(proxy) {
                client_builder = client_builder.proxy(http_proxy);
                info!("[Fetcher] Set http proxy");
            }
            if let Ok(http_proxy) = Proxy::https(proxy) {
                client_builder = client_builder.proxy(http_proxy);
                info!("[Fetcher] Set https proxy");
            }
        }
        Ok(Self {
            config,
            request_client: client_builder.build()?,
            last_update_time: Local::now().checked_sub_days(Days::new(3)).unwrap(),
        })
    }

    /// fetch feeds
    pub async fn fetch(&mut self) -> Result<Vec<(Channel, Feed)>> {
        let mut vec = vec![];
        let update_time = Local::now();
        for feed in self.config.feeds() {
            let content = self
                .request_client
                .get(feed.url.clone())
                .send()
                .await?
                .bytes()
                .await?;
            debug!("[Fetch] Getting Channel");
            let mut channel = Channel::read_from(&content[..])?;

            debug!("[Fetch] Filtering Channel");
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

            debug!("[Fetch] Filtering items");
            // filter items
            channel.items.retain(|item| {
                if let Some(pub_date) = item.pub_date() {
                    let date_time = pub_date2date_time(pub_date);
                    return date_time > self.last_update_time && date_time <= update_time;
                }
                // Note: mikan pub date is in torrent
                if let Some(torrent) = &item.torrent {
                    if let Some(pub_date) = torrent.pub_date.as_ref() {
                        let date_time = pub_date2date_time(pub_date);
                        return date_time > self.last_update_time && date_time <= update_time;
                    }
                }
                true
            });

            if channel.items().is_empty() {
                debug!("[Fetch] No new item");
                continue;
            }

            info!(
                "[Fetch] Got {} items from link: {}",
                channel.items().len(),
                channel.link()
            );
            vec.push((channel, feed.clone()));
        }
        self.last_update_time = update_time;
        Ok(vec)
    }
}

fn pub_date2date_time(pub_date: &str) -> DateTime<Local> {
    let mut pub_date = pub_date.to_string();
    pub_date.push_str("+08:00");
    DateTime::parse_from_rfc3339(pub_date.as_str())
        .unwrap()
        .with_timezone(&Local {})
}
