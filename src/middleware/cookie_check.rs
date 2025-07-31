use axum::{extract::Request, middleware::Next, response::Response, Extension};
use cookie::Cookie;
use reqwest::StatusCode;
use tracing::error;
use worker::Env;

use crate::{
    services::{
        auth::{add_success_cookies, DiscordAPIClient, DiscordCookie},
        cookie::CookieJar,
        get_discord_env,
    },
    state::{
        server_info::ServerInfoArc,
        user::{RequestedUser, User},
    },
};

#[worker::send]
pub async fn middleware(
    Extension(env): Extension<Env>,
    Extension(server_info): Extension<ServerInfoArc>,
    Extension(requested_user): Extension<RequestedUser>,
    jar: CookieJar,
    mut req: Request,
    next: Next,
) -> Result<(Option<(CookieJar, CookieJar)>, Response), (Option<(CookieJar, CookieJar)>, StatusCode)>
{
    if let RequestedUser::Bot(_) = requested_user {
        return Ok((None, next.run(req).await));
    }

    match jar
        .get(&DiscordCookie::AccessToken.to_string())
        .map(|c| c.value().to_string())
    {
        Some(token) => {
            let user = User::new(token);
            req.extensions_mut()
                .insert(RequestedUser::UserWithToken(user));
            Ok((None, next.run(req).await))
        }
        None => {
            let Some(refresh_token) = jar
                .get(&DiscordCookie::RefreshToken.to_string())
                .map(|c| c.value().to_string())
            else {
                error!("No access token or refresh token found in cookies");
                return Ok((None, next.run(req).await));
            };
            let Ok((client_id, client_secret)) = get_discord_env(&env) else {
                error!("Failed to get Discord environment variables");
                return Err((None, StatusCode::INTERNAL_SERVER_ERROR));
            };
            let redirect_uri = format!("{}/api/auth/redirect", server_info.api_host());

            let discord_api =
                DiscordAPIClient::new(client_id.clone(), client_secret.clone(), redirect_uri);

            let token = discord_api
                .refresh_access_token(&refresh_token)
                .await
                .map_err(|e| {
                    error!("Failed to refresh access token: {}", e);
                    (None, StatusCode::UNAUTHORIZED)
                })?;
            let cookies = DiscordAPIClient::set_cookies(token.clone());

            let user = User::new(token.access_token().to_string());
            req.extensions_mut()
                .insert(RequestedUser::UserWithToken(user));
            Ok((
                Some(add_success_cookies(&jar, cookies)),
                next.run(req).await,
            ))
        }
    }
}
