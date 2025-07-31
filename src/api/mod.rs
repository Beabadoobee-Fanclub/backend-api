mod auth;
mod protected;

use axum::Router;

pub fn router() -> Router {
    Router::new()
        .merge(protected::router())
        .nest("/auth", auth::router())
}
