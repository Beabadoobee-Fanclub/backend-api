use axum::{extract::Request, middleware::Next, response::Response};

pub async fn middleware(request: Request, next: Next) -> Response {
    let response = next.run(request).await;

    response
}
