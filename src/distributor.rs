use std::sync::Arc;

use anyhow::{anyhow, Context};
use crossbeam_queue::ArrayQueue;
use pikpak_api::pikpak::{self, ClientOptions};
use resend_rs::{types::CreateEmailBaseOptions, Resend};
use rss_for_mikan::{Channel, Item};

use futures::{
    future::ok,
    stream::{self, StreamExt},
};
use tokio::sync::Mutex;
use tracing::info;

use crate::{config::Config, subscriber::SubscriberSrc};

pub struct Distributor {
    config: Arc<Config>,
    resend_client: Resend,
    pikpak_client: Option<Arc<Mutex<pikpak::Client>>>,
}

impl Distributor {
    pub fn new(config: Arc<Config>) -> Self {
        let resend_client = Resend::new(config.resend_api_key());

        Distributor {
            config,
            resend_client,
            pikpak_client: None,
        }
    }

    pub async fn init_pikpak_client(&mut self) -> anyhow::Result<()> {
        info!("[Distributor] init pikpak client");

        let pikpak_config = self
            .config
            .pikpak_config()
            .ok_or(anyhow!("pikpak config not found"))?;
        let proxy = self.config.proxy().map(|s| s.to_string());
        let options = ClientOptions {
            username: pikpak_config.username.clone(),
            password: pikpak_config.password.clone(),
            retry_times: 3,
            proxy,
        };
        let mut pikpak_client =
            pikpak::Client::new(options).context("[init pikpak] create client failed")?;
        pikpak_client
            .login()
            .await
            .context("[init pikpak] login failed")?;
        self.pikpak_client = Some(Arc::new(Mutex::new(pikpak_client)));

        Ok(())
    }

    pub async fn notify(
        &mut self,
        channel: &Channel,
        sub: &SubscriberSrc,
    ) -> anyhow::Result<Vec<Item>> {
        match sub {
            SubscriberSrc::Email(email_url) => self.send_email(email_url.as_str(), channel).await,
            SubscriberSrc::PikPak => self.magnet_pikpak(channel).await,
        }
    }

    async fn send_email(&self, email_url: &str, channel: &Channel) -> anyhow::Result<Vec<Item>> {
        let from = self.config.send_email();
        let to = vec![email_url];
        let subject = format!("{} - {}", "AutoAnime", channel.title);
        // TODO: more specific
        let email = CreateEmailBaseOptions::new(from, to, subject)
            .with_text(channel.link())
            .with_html(channel2html(channel).as_str());
        self.resend_client.emails.send(email).await?;
        Ok(Vec::new())
    }

    async fn magnet_pikpak(&mut self, channel: &Channel) -> anyhow::Result<Vec<Item>> {
        // init pikpak client
        if self.pikpak_client.is_none() {
            self.init_pikpak_client().await?;
        }
        // upload torrent on pikpak
        if let Some(client) = self.pikpak_client.as_ref() {
            let stream = stream::iter(channel.items.clone());
            let config = self.config.clone();
            let queue: Arc<ArrayQueue<Item>> = Arc::new(ArrayQueue::new(channel.items.len()));

            let fut = stream.for_each_concurrent(None, |item| {
                let client = client.clone();
                let config = config.clone();
                let queue = queue.clone();
                async move {
                    if item.torrent.is_some() {
                        // torrent is in enclosure.url
                        if let Some(enclosure) = item.enclosure().as_ref() {
                            let torrent_url = enclosure.url().to_owned();
                            info!(
                                "[Pikpak] downloading anime: {}",
                                item.title().unwrap_or_default()
                            );
                            info!("[Pikpak] torrent: {}", torrent_url);

                            if let Err(e) = client
                                .lock()
                                .await
                                .new_magnet(
                                    config.pikpak_config().unwrap().path.as_str(),
                                    &torrent_url,
                                )
                                .await
                            {
                                tracing::error!(
                                    "Pikpak Download Error: anime: {}, error: {}",
                                    item.title().unwrap_or_default(),
                                    e
                                );
                                queue.push(item).ok();
                            }
                        }
                    }
                }
            });
            fut.await;
            let mut items_vec = Vec::new();
            while let Some(item) = queue.pop() {
                items_vec.push(item);
            }
            Ok(items_vec)
        } else {
            Err(anyhow!("pikpak client not started"))
        }
    }
}

fn channel2html(channel: &Channel) -> String {
    let mut html = String::new();
    html.push_str(format!("<p><b>{}</b><p>\n", channel.description()).as_str());
    html.push_str("<ul>\n");
    for item in channel.items() {
        html.push_str(
            format!(
                "<li>{} - {}</li>",
                item.description().unwrap_or(""),
                item.torrent
                    .as_ref()
                    .map(|torrent| torrent.link.as_deref().unwrap_or(""))
                    .unwrap_or("")
            )
            .as_str(),
        );
    }
    html.push_str("</ul>\n");
    html
}
