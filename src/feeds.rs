use serde::{Deserialize, Serialize};

use crate::subscriber::Subscriber;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Feed {
    // feeds url
    pub url: String,
    // subscriber
    pub subscriber: Vec<Subscriber>,
}
