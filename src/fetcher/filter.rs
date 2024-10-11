use std::path::Path;

use chrono::{DateTime, Local};
use rss_for_mikan::{Guid, Item};
use serde::{Deserialize, Serialize};

use crate::subscriber::SubscriberSrc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Key {
    title: String,
    guid: Guid,
    subscriber: SubscriberSrc,
}

impl Key {
    pub fn new(title: String, guid: Guid, subscriber: SubscriberSrc) -> Self {
        Self {
            title,
            guid,
            subscriber,
        }
    }

    pub fn from_item(item: &Item, subscriber: &SubscriberSrc) -> Self {
        Self::new(
            item.title().unwrap_or_default().to_string(),
            item.guid.clone().unwrap_or_default(),
            subscriber.clone(),
        )
    }
}


type Value = DateTime<Local>;

#[derive(Clone)]
pub struct KVStore {
    inner: sled::Db,
}

impl KVStore {
    pub fn new(path: impl AsRef<Path>) -> Self {
        let inner = sled::open(path).unwrap();
        Self { inner }
    }

    /// if key not exist, insert it and return None
    /// if key exist, return the value
    pub fn get_or_insert(&self, key: Key) -> anyhow::Result<Option<Value>> {
        let value = self.get(&key)?;
        if value.is_none() {
            let value = Local::now();
            self.insert(key, value)?;
            Ok(None)
        } else {
            Ok(value)
        }
    }

    pub fn get(&self, key: &Key) -> anyhow::Result<Option<Value>> {
        let key = bincode::serialize(&key)?;
        let value = self.inner.get(key)?;
        let value = match value {
            Some(ivec) => Some(bincode::deserialize(&ivec)?),
            None => None,
        };
        Ok(value)
    }

    pub fn insert(&self, key: Key, value: Value) -> anyhow::Result<()> {
        let key = bincode::serialize(&key)?;
        let value = bincode::serialize(&value)?;
        self.inner.insert(key, value)?;
        Ok(())
    }

    pub fn remove(&self, key: &Key) -> anyhow::Result<()> {
        let key = bincode::serialize(&key)?;
        self.inner.remove(key)?;
        Ok(())
    }
}
