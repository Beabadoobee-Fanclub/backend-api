use std::sync::Arc;

use axum::{
    body::Body,
    http::{HeaderValue, Response},
    routing::get,
    Extension, Router,
};
use reqwest::{Method, StatusCode};
use tower_http::cors::{AllowCredentials, Any, CorsLayer};
use tower_service::Service;
use tracing_subscriber::{
    fmt::{format::Pretty, time::UtcTime},
    layer::SubscriberExt,
    util::SubscriberInitExt,
};
use tracing_web::{performance_layer, MakeConsoleWriter};
use worker::{console_error, event, Context, Env, Error, HttpRequest, Result};

use crate::{
    services::database::Database,
    state::{app_state::AppState, server_info::ServerInfo},
};
pub mod durables;
pub mod middleware;
pub mod services;
pub mod state;

mod api;
mod cdn;

pub const DISCORD_API_BASE_URL: &str = "https://discord.com/api/v10";
pub const DASHBOARD_URL: &str = "http://localhost:5173";

#[event(start)]
fn start() {
    let fmt_layer = tracing_subscriber::fmt::layer()
        .json()
        .with_ansi(false)
        .with_timer(UtcTime::rfc_3339())
        .with_writer(MakeConsoleWriter);

    let perf_layer = performance_layer().with_details_from_fields(Pretty::default());
    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(perf_layer)
        .init();
}

fn cors_layer(webpage: &str) -> CorsLayer {
    let webpage_header = HeaderValue::from_str(webpage).expect("Invalid URL for CORS");
    CorsLayer::new()
        .allow_origin(webpage_header)
        .allow_methods(vec![Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_credentials(AllowCredentials::yes())
}

#[event(fetch)]
async fn fetch(req: HttpRequest, env: Env, ctx: Context) -> Result<Response<Body>> {
    console_error_panic_hook::set_once();

    let Ok(hyperdrive) = env.hyperdrive("DATABASE") else {
        console_error!("Failed to get Hyperdrive instance");
        return Ok(Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(Body::from("Internal Server Error"))
            .unwrap());
    };

    let server_info = ServerInfo::new(&env)?;

    let app_state = Arc::new(AppState {
        // Initialize your application state here
        database: Database::new(hyperdrive),
    });

    let mut app = Router::new()
        .nest("/api", api::router())
        .nest("/cdn", cdn::router())
        .route("/", get(root))
        .fallback(fallback)
        .layer(axum::middleware::from_fn(
            middleware::requested_user::middleware,
        ))
        .layer(Extension(app_state))
        .layer(Extension(env))
        .layer(Extension(server_info.clone()))
        .layer(cors_layer(&server_info.webpage()));

    Ok(app.call(req).await?)
}

async fn fallback() -> Response<Body> {
    Response::new(Body::from("Not Found"))
}

pub async fn root() -> &'static str {
    "Hello Axum!"
}
