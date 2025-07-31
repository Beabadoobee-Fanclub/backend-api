mod gateway;
mod guild;

use axum::{routing::get, Router};

use crate::middleware;

pub fn router() -> Router {
    Router::new()
        .nest("/guild", guild::router())
        .route("/gateway/{id}", get(gateway::handle_websocket))
        .layer(axum::middleware::from_fn(
            middleware::api_protect::protection_middleware,
        ))
}
