use axum::response::IntoResponse;
use serde::{Deserialize, Serialize};
use worker::console_debug;

use crate::DISCORD_API_BASE_URL;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct PartialDiscordGuild {
    pub id: String,
    pub name: String,
    pub icon: Option<String>,
    pub banner: Option<String>,
    pub owner: bool,
    pub permissions: String,
    pub features: Vec<String>,
    pub approximate_member_count: Option<u64>,
    pub approximate_presence_count: Option<u64>,
}

impl IntoResponse for PartialDiscordGuild {
    fn into_response(self) -> axum::response::Response {
        let body = serde_json::to_string(&self).unwrap_or_else(|_| "{}".to_string());
        axum::response::Response::builder()
            .header("Content-Type", "application/json")
            .body(axum::body::Body::from(body))
            .unwrap()
    }
}

pub struct DiscordGuildHTTP {
    client: reqwest::Client,
}

impl DiscordGuildHTTP {
    pub fn new(authorization: String) -> Self {
        let client = reqwest::Client::builder()
            .default_headers({
                let mut headers = reqwest::header::HeaderMap::new();
                headers.insert(
                    reqwest::header::AUTHORIZATION,
                    authorization.parse().unwrap(),
                );
                headers
            })
            .build()
            .unwrap();

        Self { client }
    }

    pub async fn get_guilds(&self) -> Result<Vec<PartialDiscordGuild>, String> {
        let url = format!("{}/users/@me/guilds", DISCORD_API_BASE_URL);
        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| e.to_string())?;
        if response.status().is_success() {
            let guilds = response
                .json::<Vec<PartialDiscordGuild>>()
                .await
                .map_err(|e| e.to_string())?;
            Ok(guilds)
        } else {
            Err(format!("Failed to fetch guilds: {}", response.status()))
        }
    }

    pub async fn get_mutual_guilds(&self, other: Self) -> Result<Vec<PartialDiscordGuild>, String> {
        let self_guilds = self.get_guilds().await?;
        let other_guilds = other.get_guilds().await?;
        let mutual_guilds = self_guilds
            .into_iter()
            .filter(|guild| other_guilds.iter().any(|g| g.id == guild.id))
            .collect();
        Ok(mutual_guilds)
    }
}
