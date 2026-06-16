use moka::sync::Cache;
use reqwest::Client;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::vec;

pub struct TwitchBadgeParser {
    cache: Cache<String, Arc<CachedBadge>>,
    channel_index: Mutex<HashMap<String, Vec<String>>>,
    token: String,
    client_id: String,
    client: Client,
}

#[derive(Clone, Debug)]
pub struct CachedBadge {
    badge: TwitchBadge,
    source: BadgeSource,
}

#[derive(Clone, Debug, Deserialize)]
pub struct TwitchBadge {
    set_id: String,
    versions: Vec<EmoteVersion>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct EmoteVersion {
    id: String,
    image_url_1x: String,
    image_url_2x: String,
    image_url_4x: String,
    title: String,
    description: String,
    click_action: Option<String>,
    click_url: Option<String>,
}

#[derive(Clone, Debug)]
pub enum BadgeSource {
    Global,
    Channel(String),
}

#[derive(Clone, Debug, Deserialize)]
pub struct BadgeResponse {
    pub data: Vec<TwitchBadge>,
}

impl TwitchBadgeParser {
    pub async fn new(token: &str, client_id: &str) -> Result<Self, reqwest::Error> {
        let client = Client::new();

        let response = client
            .get("https://api.twitch.tv/helix/chat/badges/global")
            .header("Authorization", format!("Bearer {}", token))
            .header("Client-ID", client_id)
            .send()
            .await?
            .error_for_status()?;

        let parsed: BadgeResponse = response.json().await?;

        let cache = Cache::builder()
            .max_capacity(10_000)
            .time_to_live(Duration::from_secs(60 * 60 * 12))
            .build();

        for badge in parsed.data {
            let key = badge.set_id.clone();

            cache.insert(
                key,
                Arc::new(CachedBadge {
                    badge,
                    source: BadgeSource::Global,
                }),
            );
        }

        Ok(Self {
            cache,
            channel_index: Mutex::new(HashMap::new()),
            token: token.to_string(),
            client_id: client_id.to_string(),
            client,
        })
    }

    pub fn get(&self, key: &str) -> Option<Arc<CachedBadge>> {
        self.cache.get(key)
    }

    pub async fn add_channel(&self, channel_id: &str) -> Result<(), reqwest::Error> {
        let response = self
            .client
            .get("https://api.twitch.tv/helix/chat/badges")
            .query(&[("broadcaster_id", channel_id)])
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Client-ID", &self.client_id)
            .send()
            .await?
            .error_for_status()?;

        let parsed: BadgeResponse = response.json().await?;

        let mut names = Vec::new();

        for badge in parsed.data {
            let name = badge.set_id.clone();

            self.cache.insert(
                name.clone(),
                Arc::new(CachedBadge {
                    badge,
                    source: BadgeSource::Channel(channel_id.to_string()),
                }),
            );

            names.push(name);
        }

        self.channel_index
            .lock()
            .unwrap()
            .insert(channel_id.to_string(), names);

        Ok(())
    }

    pub fn remove_channel(&self, channel_id: &str) {
        if let Some(names) = self.channel_index.lock().unwrap().remove(channel_id) {
            for name in names {
                self.cache.invalidate(&name);
            }
        }
    }
}
