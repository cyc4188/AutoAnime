use serde::{Deserialize, Serialize};

use crate::feeds::Feed;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    // resend api key
    resend_api_key: String,
    // send email
    send_email: String,
    // feeds
    feeds: Vec<Feed>,
    // pikpak
    pikpak: Option<PikpakConfig>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PikpakConfig {
    username: String,
    password: String,
    path: String,
}

impl Config {
    pub fn feeds(&self) -> &Vec<Feed> {
        &self.feeds
    }
    pub fn resend_api_key(&self) -> &str {
        &self.resend_api_key
    }
    pub fn send_email(&self) -> &str {
        &self.send_email
    }
}

#[cfg(test)]
mod tests {
    use super::Config;

    #[test]
    fn test_config_yml() {
        let yaml = "
        resend_api_key: key_123
        send_email: test@test.cc
        feeds:
        - url: http://example.com
          subscriber:
            - !Email test@receiver.cc
        - url: http://example2.com
          subscriber:
            - !PikPak
        ";
        let config: Config = serde_yaml::from_str(yaml).unwrap();
        assert!(config.resend_api_key == "key_123");
        assert!(config.send_email == "test@test.cc");
        assert!(config.feeds.len() == 2);
    }
}
