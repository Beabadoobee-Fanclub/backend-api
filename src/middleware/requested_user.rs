use crate::state::user::{Bot, RequestedUser};
use axum::{extract::Request, http::HeaderMap, middleware::Next, response::Response};
use tracing::debug;

pub async fn middleware(headers: HeaderMap, mut req: Request, next: Next) -> Response {
    if let Some(client) = get_client(&headers) {
        if let ["DiscordBot", token] = client.split_whitespace().collect::<Vec<&str>>().as_slice() {
            // TODO: Validate the bot token here
            debug!("Discord Bot token: {}", token);
            let val = Bot::new(token.to_string());
            req.extensions_mut().insert(RequestedUser::Bot(val));
            return next.run(req).await;
        }
    }
    req.extensions_mut().insert(RequestedUser::User); // This is a discord bot making the request (Check if the token is valid)
    next.run(req).await
}

fn get_client(headers: &HeaderMap) -> Option<String> {
    headers
        .get("client")
        .and_then(|value| value.to_str().ok())
        .map(|s| s.to_string())
}
