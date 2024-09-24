mod filter;

use std::{path::Path, sync::Arc, time::Duration};

use anyhow::Result;
use chrono::{DateTime, Local};
use reqwest::{Client, Proxy};
use rss_for_mikan::Channel;
use tracing::{debug, error, info};

use crate::{
    config::{Config, DEFAULT_HISTORY_PATH},
    subscriber::SubscriberSrc,
};
use filter::{KVStore, Key};

pub struct Fetcher {
    config: Arc<Config>,
    request_client: Client,
    fetch_history: KVStore,
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
        let fetch_history = KVStore::new(
            config
                .history_path()
                .unwrap_or(Path::new(DEFAULT_HISTORY_PATH)),
        );
        Ok(Self {
            config,
            request_client: client_builder.build()?,
            fetch_history,
        })
    }

    /// fetch feeds
    pub async fn fetch(&mut self) -> Result<Vec<(Channel, SubscriberSrc)>> {
        let mut vec = vec![];
        for subscriber in self.config.subscriber() {
            for feed in &subscriber.feeds {
                debug!("[Fetch] Getting Channel");
                // get channel
                let content = self
                    .request_client
                    .get(feed.url.clone())
                    .send()
                    .await?
                    .bytes()
                    .await?;
                let mut channel = Channel::read_from(&content[..])?;

                debug!("[Fetch] Filtering items");
                // filter items
                channel.items.retain(|item| {
                    let key = Key::new(
                        item.title.clone().unwrap_or_default(),
                        item.guid.clone().unwrap_or_default(),
                        subscriber.src.clone(),
                    );
                    match self.fetch_history.get_or_insert(key) {
                        Ok(None) => true,
                        Ok(Some(_)) => false,
                        Err(e) => {
                            error!("[Fetch] Failed to get or insert item: {}", e);
                            false
                        }
                    }
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
                vec.push((channel, subscriber.src.clone()));
            }
        }
        Ok(vec)
    }
}

fn _pub_date2date_time(pub_date: &str) -> DateTime<Local> {
    let mut pub_date = pub_date.to_string();
    pub_date.push_str("+08:00");
    DateTime::parse_from_rfc3339(pub_date.as_str())
        .unwrap()
        .with_timezone(&Local {})
}