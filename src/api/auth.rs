use std::collections::HashMap;

use axum::{
    extract::Query,
    http::StatusCode,
    response::Redirect,
    routing::{get, post},
    Extension, Json, Router,
};
use cookie::{time::Duration, Cookie};
use worker::{console_error, Env};

use crate::{
    services::{
        auth::{DiscordAPIClient, DiscordOAuth2, DiscordOAuth2Scope},
        cookie::CookieJar,
        get_discord_env,
        user::{DiscordUser, DiscordUserApi},
    },
    DASHBOARD_URL,
};

pub fn router() -> Router {
    Router::new()
        .route("/login", get(login))
        .route("/redirect", get(redirect))
        .route("/status", get(status))
        .route("/logout", get(logout))
}

async fn login(Extension(env): Extension<Env>, jar: CookieJar) -> Redirect {
    let webpage = env
        .var("DASHBOARD_URL")
        .map(|s| s.to_string())
        .unwrap_or_else(|_| DASHBOARD_URL.into());
    let Ok((client_id, _)) = get_discord_env(&env) else {
        console_error!("Failed to get Discord environment variables");
        return Redirect::to(&webpage);
    };

    match jar.get("discord_token") {
        Some(_) => {
            let dashboard = format!("{}/dashboard", webpage);
            console_error!("User is already logged in, redirecting to dashboard");
            Redirect::to(&dashboard)
        }
        None => {
            let discord_oauth = DiscordOAuth2 {
                client_id,
                redirect_uri: "http://127.0.0.1:8787/api/auth/redirect".to_string(),
                scopes: vec![
                    DiscordOAuth2Scope::Identify,
                    DiscordOAuth2Scope::Guilds,
                    DiscordOAuth2Scope::Email,
                ],
            };

            let discord_url = discord_oauth.get_url();
            console_error!("Redirecting to Discord OAuth2 login");
            Redirect::temporary(discord_url.as_ref())
        }
    }
}

#[worker::send]
async fn redirect(
    Extension(env): Extension<Env>,
    Query(params): Query<HashMap<String, String>>,
    jar: CookieJar,
) -> Result<(CookieJar, CookieJar, Redirect), Redirect> {
    let webpage = env
        .var("DASHBOARD_URL")
        .map(|s| s.to_string())
        .unwrap_or_else(|_| DASHBOARD_URL.into());
    let dashboard = format!("{}/dashboard", webpage);

    let Ok((client_id, client_secret)) = get_discord_env(&env) else {
        console_error!("Failed to get Discord environment variables");
        return Err(Redirect::temporary(&webpage));
    };

    let redirect_uri = "http://127.0.0.1:8787/api/auth/redirect".to_string();
    let code = match params.get("code") {
        Some(code) => code,
        None => {
            console_error!("No code provided in redirect");
            return Err(Redirect::temporary(&webpage));
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
            console_error!("Failed to get access token: {}", e);
            return Err(Redirect::to(&webpage));
        }
    };

    let cookies = DiscordAPIClient::set_cookies(token);

    Ok((
        jar.clone().add(cookies[0].clone()),
        jar.clone().add(cookies[1].clone()),
        Redirect::to(&dashboard),
    ))
}

#[axum::debug_handler]
#[worker::send]
async fn status(
    Extension(env): Extension<Env>,
    jar: CookieJar,
) -> Result<
    (Option<(CookieJar, CookieJar)>, Json<DiscordUser>),
    (Option<(CookieJar, CookieJar)>, StatusCode),
> {
    let (token, cookies) = match jar.get("discord_token").map(|c| c.value().to_string()) {
        Some(token) => (token, None),
        None => {
            let Ok((client_id, client_secret)) = get_discord_env(&env) else {
                console_error!("Failed to get Discord environment variables");
                return Err((None, StatusCode::INTERNAL_SERVER_ERROR));
            };
            let Some(refresh_token) = jar
                .get("discord_refresh_token")
                .map(|c| c.value().to_string())
            else {
                console_error!("No access token or refresh token found in cookies");
                return Err((None, StatusCode::UNAUTHORIZED));
            };

            let redirect_uri = "http://127.0.0.1:8787/api/auth/redirect".to_string();

            let discord_api =
                DiscordAPIClient::new(client_id.clone(), client_secret.clone(), redirect_uri);

            let token = discord_api
                .refresh_access_token(&refresh_token)
                .await
                .map_err(|e| {
                    console_error!("Failed to refresh access token: {}", e);
                    (None, StatusCode::UNAUTHORIZED)
                })?;
            let cookies = DiscordAPIClient::set_cookies(token.clone());

            (
                token.access_token().to_string(),
                Some(add_success_cookies(&jar, cookies)),
            )
        }
    };
    let authorization = format!("Bearer {}", token);
    let discord_user_api = DiscordUserApi::new(authorization);
    let user = match discord_user_api.get_user().await {
        Ok(user) => user,
        Err(e) => {
            console_error!("Failed to fetch user data: {}", e);
            return Err((Some(remove_error_cookies(&jar)), StatusCode::UNAUTHORIZED));
        }
    };

    Ok((cookies, Json(user)))
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

fn remove_error_cookies(jar: &CookieJar) -> (CookieJar, CookieJar) {
    let discord_token = Cookie::build(("discord_token", ""))
        .path("/")
        .http_only(true)
        .max_age(Duration::ZERO)
        .build();
    let discord_refresh_token = Cookie::build(("discord_refresh_token", ""))
        .path("/")
        .http_only(true)
        .max_age(Duration::ZERO)
        .build();
    (
        jar.clone().add(discord_token),
        jar.clone().add(discord_refresh_token),
    )
}

fn add_success_cookies(jar: &CookieJar, cookies: [Cookie<'static>; 2]) -> (CookieJar, CookieJar) {
    (
        jar.clone().add(cookies[0].clone()),
        jar.clone().add(cookies[1].clone()),
    )
}
