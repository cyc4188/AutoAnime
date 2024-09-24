use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Feed {
    // feeds url
    pub url: String,
}
