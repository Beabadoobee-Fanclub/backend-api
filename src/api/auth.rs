use std::{collections::HashMap, sync::Arc};

use axum::{
    extract::Query,
    http::StatusCode,
    response::Redirect,
    routing::{get, post},
    Extension, Json, Router,
};
use cookie::{time::Duration, Cookie};
use tracing::{error, info, warn};
use worker::{console_error, console_log, Env};

use crate::{
    services::{
        auth::{remove_error_cookies, DiscordAPIClient, DiscordOAuth2, DiscordOAuth2Scope},
        cookie::CookieJar,
        get_discord_env,
        user::{DiscordUser, DiscordUserApi},
    },
    state::{server_info::ServerInfoArc, user::RequestedUser},
    DASHBOARD_URL,
};

pub fn router() -> Router {
    Router::new()
        .route("/login", get(login))
        .route("/redirect", get(redirect))
        .route("/status", get(status))
        .route("/logout", get(logout))
}

async fn login(
    Extension(env): Extension<Env>,
    Extension(server_info): Extension<ServerInfoArc>,
    Extension(requested_user): Extension<RequestedUser>,
) -> Result<Redirect, StatusCode> {
    let Ok((client_id, _)) = get_discord_env(&env) else {
        error!("Failed to get Discord environment variables");
        return Ok(Redirect::to(server_info.webpage()));
    };

    if let RequestedUser::Bot(_) = requested_user {
        warn!("Bots cannot log in through the web interface");
        return Err(StatusCode::FORBIDDEN);
    }

    if let RequestedUser::UserWithToken(_) = requested_user {
        let dashboard = format!("{}/dashboard", server_info.webpage());
        warn!("User is already logged in, redirecting to dashboard");
        return Ok(Redirect::to(&dashboard));
    }

    let redirect = format!("{}/api/auth/redirect", server_info.api_host());
    let discord_oauth = DiscordOAuth2 {
        client_id,
        redirect_uri: redirect,
        scopes: vec![
            DiscordOAuth2Scope::Identify,
            DiscordOAuth2Scope::Guilds,
            DiscordOAuth2Scope::Email,
        ],
    };

    let discord_url = discord_oauth.get_url();
    info!("Redirecting to Discord OAuth2 login");
    Ok(Redirect::temporary(discord_url.as_ref()))
}

#[worker::send]
async fn redirect(
    Extension(env): Extension<Env>,
    Extension(server_info): Extension<ServerInfoArc>,
    Query(params): Query<HashMap<String, String>>,
    jar: CookieJar,
) -> Result<(CookieJar, CookieJar, Redirect), Redirect> {
    let webpage = server_info.webpage();
    let dashboard = format!("{}/dashboard", webpage);

    let Ok((client_id, client_secret)) = get_discord_env(&env) else {
        error!("Failed to get Discord environment variables");
        return Err(Redirect::temporary(webpage));
    };

    let redirect_uri = format!("{}/api/auth/redirect", server_info.api_host());
    let code = match params.get("code") {
        Some(code) => code,
        None => {
            error!("No code provided in redirect");
            return Err(Redirect::temporary(webpage));
        }
    };

    let discord_api = DiscordAPIClient::new(
        client_id.clone(),
        client_secret.clone(),
        redirect_uri.clone(),
    );
    let token = match discord_api.get_access_token(code.clone()).await {
        Ok(token) => token,
        Err(e) => {
            error!("Failed to get access token: {}", e);
            return Err(Redirect::to(webpage));
        }
    };

    let cookies = DiscordAPIClient::set_cookies(token);

    Ok((
        jar.clone().add(cookies[0].clone()),
        jar.clone().add(cookies[1].clone()),
        Redirect::to(&dashboard),
    ))
}

#[worker::send]
async fn status(
    Extension(requested_user): Extension<RequestedUser>,
    jar: CookieJar,
) -> Result<Json<DiscordUser>, (Option<(CookieJar, CookieJar)>, StatusCode)> {
    let user = match requested_user {
        RequestedUser::UserWithToken(user) => user,
        _ => {
            error!("Unauthorized access to status endpoint");
            return Err((None, StatusCode::UNAUTHORIZED));
        }
    };

    let authorization = format!("Bearer {}", user.access_token());
    let discord_user_api = DiscordUserApi::new(authorization);
    let user = match discord_user_api.get_user().await {
        Ok(user) => user,
        Err(e) => {
            error!("Failed to fetch user data: {}", e);
            return Err((Some(remove_error_cookies(&jar)), StatusCode::UNAUTHORIZED));
        }
    };

    Ok(Json(user))
}

async fn logout(
    Extension(env): Extension<Env>,
    jar: CookieJar,
) -> ((CookieJar, CookieJar), Redirect) {
    let webpage = env
        .var("DASHBOARD_URL")
        .map(|s| s.to_string())
        .unwrap_or_else(|_| DASHBOARD_URL.into());
    (remove_error_cookies(&jar), Redirect::to(&webpage))
}
