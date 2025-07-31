mod auth;
mod guilds;
mod protected;

use crate::middleware;
use axum::Router;

pub fn router() -> Router {
    Router::new()
        .merge(protected::router())
        .nest("/guilds", guilds::router())
        .nest("/auth", auth::router())
        .layer(axum::middleware::from_fn(
            middleware::cookie_check::middleware,
        ))
}
