use serde::{Deserialize, Serialize};

use crate::feeds::Feed;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Subscriber {
    pub src: SubscriberSrc,
    pub feeds: Vec<Feed>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum SubscriberSrc {
    #[serde(rename = "email")]
    Email(String),
    #[serde(rename = "pikpak")]
    PikPak,
}

impl SubscriberSrc {
    pub fn is_pikpak(&self) -> bool {
        matches!(self, SubscriberSrc::PikPak)
    }
    pub fn is_email(&self) -> bool {
        matches!(self, SubscriberSrc::Email(_))
    }
}
