use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::subscriber::Subscriber;

pub const CONFIG_PATH: &str = "./config.yaml";
pub const DEFAULT_HISTORY_PATH: &str = "./history.sled";

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    // resend api key
    resend_api_key: String,
    // send email
    send_email: String,
    // subscriber
    subscriber: Vec<Subscriber>,
    // pikpak
    pikpak: Option<PikpakConfig>,
    // proxy
    proxy: Option<String>,
    // time config
    frequency: Option<FrequencyConfig>,
    // history path
    history_path: Option<PathBuf>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum FrequencyConfig {
    #[serde(rename = "minutely")]
    Minutely(u64),
    #[serde(rename = "hourly")]
    Hourly(u64),
    #[serde(rename = "daily")]
    Daily(u64),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PikpakConfig {
    pub username: String,
    pub password: String,
    pub path: String,
}

impl Config {
    pub fn subscriber(&self) -> &Vec<Subscriber> {
        &self.subscriber
    }
    pub fn resend_api_key(&self) -> &str {
        &self.resend_api_key
    }
    pub fn send_email(&self) -> &str {
        &self.send_email
    }
    pub fn proxy(&self) -> Option<&str> {
        self.proxy.as_deref()
    }
    pub fn pikpak_config(&self) -> Option<&PikpakConfig> {
        self.pikpak.as_ref()
    }
    pub fn frequency(&self) -> Option<&FrequencyConfig> {
        self.frequency.as_ref()
    }
    pub fn history_path(&self) -> Option<&Path> {
        self.history_path.as_deref()
    }
}
pub fn get_config(path: &str) -> anyhow::Result<Config> {
    let config: Config = serde_yaml::from_str(std::fs::read_to_string(path)?.as_str())?;
    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::Config;

    #[test]
    fn test_config_yml() {
        let yaml = "
        resend_api_key: key_123
        send_email: test@test.cc
        subscriber:
        - src: !email example@qq.com
          feeds:
          - url: http://example2.com
          - url: http://example2.com
        frequency:
            !minutely 1
        ";
        let config: Config = serde_yaml::from_str(yaml).unwrap();
        assert!(config.resend_api_key == "key_123");
        assert!(config.send_email == "test@test.cc");
        assert!(config.subscriber.len() == 1);
    }
}
