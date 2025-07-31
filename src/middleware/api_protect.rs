use axum::{extract::Request, http::HeaderMap, middleware::Next, response::Response, Extension};
use reqwest::{header::USER_AGENT, StatusCode};
use worker::Env;

use crate::services::cookie::CookieJar;

pub async fn protection_middleware(
    Extension(env): Extension<Env>,
    headers: HeaderMap,
    jar: CookieJar,
    request: Request,
    next: Next,
) -> Response {
    let Some(user_agent) = get_user_agent(&headers) else {
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body("User-Agent header is required".into())
            .unwrap();
    };

    match user_agent
        .split_whitespace()
        .collect::<Vec<&str>>()
        .as_slice()
    {
        ["DiscordBot", token] => {
            // Handle Discord bot requests
        }
        ["DiscordGuild", guild_id] => {
            // Handle Discord guild requests
        }
        _ => {
            return Response::builder()
                .status(StatusCode::UNAUTHORIZED)
                .body("Unauthorized user agent".into())
                .unwrap();
        }
    }

    let response = next.run(request).await;

    response
}

fn get_user_agent(headers: &HeaderMap) -> Option<String> {
    headers
        .get(USER_AGENT)
        .and_then(|value| value.to_str().ok())
        .map(|s| s.to_string())
}
