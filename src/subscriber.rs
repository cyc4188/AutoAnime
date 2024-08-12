use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
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
}
