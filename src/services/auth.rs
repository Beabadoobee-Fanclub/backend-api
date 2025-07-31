use cookie::{Cookie, SameSite};
use reqwest::ClientBuilder;
use serde::{Deserialize, Serialize};
use worker::{console_error, Result, Url};

use crate::DISCORD_API_BASE_URL;

pub enum DiscordOAuth2Scope {
    Identify,
    Guilds,
    Email,
    GuildsChannelsRead,
    Rpc,
    RpcVoiceWrite,
    RpcScreenshareRead,
    ApplicationsBuildsRead,
    WebhookIncoming,
    ApplicationsEntitlements,
    ActivitiesInvitesWrite,
    Voice,
    DmChannelsMessagesRead,
    PresencesRead,
    AccountGlobalNameUpdate,
    SdkSocialLayer,
    ApplicationsCommandsPermissionsUpdate,
    LobbiesWrite,
    DmChannelsMessagesWrite,
    PresencesWrite,
    PaymentSourcesCountryCode,
    DmChannelsRead,
    RelationshipsRead,
    ActivitiesRead,
    MessagesRead,
    RpcScreenshareWrite,
    RpcVideoRead,
    ApplicationsCommands,
    RpcNotificationsRead,
    GdmJoin,
    GuildsJoin,
    GuildsMembersRead,
    Connections,
    Bot,
    RpcVoiceRead,
    RpcVideoWrite,
    RpcActivitiesWrite,
    ApplicationsBuildsUpload,
    ApplicationsStoreUpdate,
    ActivitiesWrite,
    RelationshipsWrite,
    RoleConnectionsWrite,
    Openid,
}

impl std::fmt::Display for DiscordOAuth2Scope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            DiscordOAuth2Scope::Identify => "identify",
            DiscordOAuth2Scope::Guilds => "guilds",
            DiscordOAuth2Scope::Email => "email",
            DiscordOAuth2Scope::GuildsChannelsRead => "guilds.channels.read",
            DiscordOAuth2Scope::Rpc => "rpc",
            DiscordOAuth2Scope::RpcVoiceWrite => "rpc.voice.write",
            DiscordOAuth2Scope::RpcScreenshareRead => "rpc.screenshare.read",
            DiscordOAuth2Scope::ApplicationsBuildsRead => "applications.builds.read",
            DiscordOAuth2Scope::WebhookIncoming => "webhook.incoming",
            DiscordOAuth2Scope::ApplicationsEntitlements => "applications.entitlements",
            DiscordOAuth2Scope::ActivitiesInvitesWrite => "activities.invites.write",
            DiscordOAuth2Scope::Voice => "voice",
            DiscordOAuth2Scope::DmChannelsMessagesRead => "dm_channels.messages.read",
            DiscordOAuth2Scope::PresencesRead => "presences.read",
            DiscordOAuth2Scope::AccountGlobalNameUpdate => "account.global_name.update",
            DiscordOAuth2Scope::SdkSocialLayer => "sdk.social_layer",
            DiscordOAuth2Scope::ApplicationsCommandsPermissionsUpdate => {
                "applications.commands.permissions.update"
            }
            DiscordOAuth2Scope::LobbiesWrite => "lobbies.write",
            DiscordOAuth2Scope::DmChannelsMessagesWrite => "dm_channels.messages.write",
            DiscordOAuth2Scope::PresencesWrite => "presences.write",
            DiscordOAuth2Scope::PaymentSourcesCountryCode => "payment_sources.country_code",
            DiscordOAuth2Scope::DmChannelsRead => "dm_channels.read",
            DiscordOAuth2Scope::RelationshipsRead => "relationships.read",
            DiscordOAuth2Scope::ActivitiesRead => "activities.read",
            DiscordOAuth2Scope::MessagesRead => "messages.read",
            DiscordOAuth2Scope::RpcScreenshareWrite => "rpc.screenshare.write",
            DiscordOAuth2Scope::RpcVideoRead => "rpc.video.read",
            DiscordOAuth2Scope::ApplicationsCommands => "applications.commands",
            DiscordOAuth2Scope::RpcNotificationsRead => "rpc.notifications.read",
            DiscordOAuth2Scope::GdmJoin => "gdm.join",
            DiscordOAuth2Scope::GuildsJoin => "guilds.join",
            DiscordOAuth2Scope::GuildsMembersRead => "guilds.members.read",
            DiscordOAuth2Scope::Connections => "connections",
            DiscordOAuth2Scope::Bot => "bot",
            DiscordOAuth2Scope::RpcVoiceRead => "rpc.voice.read",
            DiscordOAuth2Scope::RpcVideoWrite => "rpc.video.write",
            DiscordOAuth2Scope::RpcActivitiesWrite => "rpc.activities.write",
            DiscordOAuth2Scope::ApplicationsBuildsUpload => "applications.builds.upload",
            DiscordOAuth2Scope::ApplicationsStoreUpdate => "applications.store.update",
            DiscordOAuth2Scope::ActivitiesWrite => "activities.write",
            DiscordOAuth2Scope::RelationshipsWrite => "relationships.write",
            DiscordOAuth2Scope::RoleConnectionsWrite => "role_connections.write",
            DiscordOAuth2Scope::Openid => "openid",
        };
        write!(f, "{}", s)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscordOAuthAccessToken {
    access_token: String,
    refresh_token: String,
    token_type: String,
    expires_in: i64,
    scope: String,
}

impl DiscordOAuthAccessToken {
    pub fn access_token(&self) -> &str {
        &self.access_token
    }

    pub fn refresh_token(&self) -> &str {
        &self.refresh_token
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DiscordOAuthGrantType {
    #[serde(rename = "authorization_code")]
    AuthorizationCode,
    #[serde(rename = "refresh_token")]
    RefreshToken,
}

#[derive(Debug, Clone, Serialize, Deserialize)]

pub struct DiscordAccessCodeBody {
    client_id: String,
    client_secret: String,
    grant_type: DiscordOAuthGrantType,
    #[serde(skip_serializing_if = "Option::is_none")]
    code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    refresh_token: Option<String>,
    redirect_uri: String,
}

pub struct DiscordOAuth2 {
    pub client_id: String,
    pub redirect_uri: String,
    pub scopes: Vec<DiscordOAuth2Scope>,
}

impl DiscordOAuth2 {
    pub fn get_url(&self) -> Url {
        let discord_url = format!("{}/oauth2/authorize", DISCORD_API_BASE_URL);
        let mut discord_url = Url::parse(&discord_url).unwrap();
        let scope_string = self
            .scopes
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>()
            .join("+");

        // Manually build the query string to avoid encoding the '+' in scope
        let query = format!(
            "client_id={}&response_type=code&redirect_uri={}&scope={}",
            &self.client_id,
            urlencoding::encode(&self.redirect_uri),
            scope_string // do not encode scope_string
        );

        discord_url.set_query(Some(&query));
        discord_url
    }
}

pub struct DiscordAPIClient {
    client: reqwest::Client,
    client_id: String,
    client_secret: String,
    redirect_uri: String,
}

impl DiscordAPIClient {
    pub fn new(
        discord_client_id: String,
        discord_client_secret: String,
        redirect_uri: String,
    ) -> Self {
        let client = ClientBuilder::new();

        Self {
            client: client.build().expect("Failed to build reqwest client"),
            client_id: discord_client_id,
            client_secret: discord_client_secret,
            redirect_uri,
        }
    }

    pub async fn get_access_token(&self, code: String) -> Result<DiscordOAuthAccessToken> {
        let url = format!("{}/oauth2/token", DISCORD_API_BASE_URL);
        let params = DiscordAccessCodeBody {
            client_id: self.client_id.clone(),
            client_secret: self.client_secret.clone(),
            grant_type: DiscordOAuthGrantType::AuthorizationCode,
            code: Some(code),
            refresh_token: None,
            redirect_uri: self.redirect_uri.clone(),
        };

        let response = match self.client.post(&url).form(&params).send().await {
            Ok(resp) => resp,
            Err(e) => {
                console_error!("Error sending request to Discord API: {}", e);
                return Err(worker::Error::RustError(
                    "Failed to send request to Discord API".into(),
                ));
            }
        };

        let token = match response.json::<DiscordOAuthAccessToken>().await {
            Ok(token) => token,
            Err(e) => {
                console_error!("Error parsing response from Discord API: {}", e);
                return Err(worker::Error::RustError(
                    "Failed to parse response from Discord API".into(),
                ));
            }
        };

        Ok(token)
    }

    pub async fn refresh_access_token(&self, code: &str) -> Result<DiscordOAuthAccessToken> {
        let url = format!("{}/oauth2/token", DISCORD_API_BASE_URL);
        let params = DiscordAccessCodeBody {
            client_id: self.client_id.to_string(),
            client_secret: self.client_secret.to_string(),
            grant_type: DiscordOAuthGrantType::RefreshToken,
            code: None,
            refresh_token: Some(code.to_string()),
            redirect_uri: self.redirect_uri.to_string(),
        };

        let response = match self.client.post(&url).form(&params).send().await {
            Ok(resp) => resp,
            Err(e) => {
                console_error!("Error sending request to Discord API: {}", e);
                return Err(worker::Error::RustError(
                    "Failed to send request to Discord API".into(),
                ));
            }
        };

        let token = match response.json::<DiscordOAuthAccessToken>().await {
            Ok(token) => token,
            Err(e) => {
                console_error!("Error parsing response from Discord API: {}", e);
                return Err(worker::Error::RustError(
                    "Failed to parse response from Discord API".into(),
                ));
            }
        };

        Ok(token)
    }

    pub fn set_cookies(tokens: DiscordOAuthAccessToken) -> [Cookie<'static>; 2] {
        let access_cookie = Cookie::build(("discord_token", tokens.access_token.clone()))
            .path("/")
            .http_only(true)
            .secure(true)
            .same_site(SameSite::None)
            .max_age(cookie::time::Duration::seconds(tokens.expires_in))
            .build();

        let refresh_cookie = Cookie::build(("discord_refresh_token", tokens.refresh_token.clone()))
            .path("/")
            .http_only(true)
            .secure(true)
            .same_site(SameSite::None)
            .build();

        [access_cookie, refresh_cookie]
    }
}
