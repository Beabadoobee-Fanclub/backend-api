use std::fmt::Error;

use axum::response::IntoResponse;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscordUser {
    pub id: String,
    pub username: String,
    pub discriminator: String,
    pub global_name: Option<String>,
    pub bot: Option<bool>,
    pub avatar: Option<String>,
    pub verified: bool,
    pub email: Option<String>,
    pub flags: u64,
    pub banner: Option<String>,
    pub accent_color: Option<u32>,
    pub premium_type: u8,
    pub public_flags: u64,
}

impl IntoResponse for DiscordUser {
    fn into_response(self) -> axum::response::Response {
        let body = serde_json::to_string(&self).unwrap_or_else(|_| "{}".to_string());
        axum::response::Response::builder()
            .header("Content-Type", "application/json")
            .body(axum::body::Body::from(body))
            .unwrap()
    }
}

pub struct DiscordUserApi {
    client: reqwest::Client,
}

impl DiscordUserApi {
    pub fn new(authorization: String) -> Self {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::AUTHORIZATION,
            authorization.parse().unwrap(),
        );
        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .expect("Failed to create HTTP client");

        Self { client }
    }

    pub async fn get_user(&self) -> Result<DiscordUser, Error> {
        let url = format!("{}/users/@me", crate::DISCORD_API_BASE_URL);
        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| panic!("Failed to send request to Discord API: {}", e))?;

        if response.status().is_success() {
            let user: DiscordUser = response
                .json()
                .await
                .map_err(|e| panic!("Failed to parse user data: {}", e))?;
            Ok(user)
        } else {
            panic!("Failed to fetch user data: {}", response.status())
        }
    }
}
