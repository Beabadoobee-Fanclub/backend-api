use axum::{routing::get, Router};

pub fn router() -> Router {
    Router::new().route("/", get(get_guilds))
}

async fn get_guilds() -> &'static str {
    "List of guilds"
}
