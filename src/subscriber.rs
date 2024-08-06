use resend_rs::{types::CreateEmailBaseOptions, Resend};
use rss_for_mikan::Channel;
use serde::{Deserialize, Serialize};

use crate::config::Config;

#[derive(Debug, Serialize, Deserialize)]
pub enum Subscriber {
    #[serde(rename = "email")]
    Email(String),
    #[serde(rename = "pikpak")]
    PikPak,
}

impl Subscriber {
    pub fn is_pikpak(&self) -> bool {
        matches!(self, Subscriber::PikPak)
    }
    pub fn is_email(&self) -> bool {
        matches!(self, Subscriber::Email(_))
    }

    pub async fn notify(&self, config: &Config, channel: &Channel) -> anyhow::Result<()> {
        match self {
            Subscriber::Email(email_url) => {
                let resend_client = Resend::new(config.resend_api_key());
                let from = config.send_email();
                let to = vec![email_url.as_str()];
                let subject = format!("{} - {}", "AutoAnime", channel.title);
                // TODO: more specific
                let email = CreateEmailBaseOptions::new(from, to, subject)
                    .with_text(channel.link())
                    .with_html(channel.description());
                resend_client.emails.send(email).await?;
            }
            Subscriber::PikPak => {
                todo!();
            }
        }
        Ok(())
    }
}
