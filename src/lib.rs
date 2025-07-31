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
use worker::{console_error, event, Context, Env, HttpRequest, Result};

use crate::services::database::Database;
pub mod durables;
pub mod middleware;
pub mod services;

mod api;
mod cdn;

pub const DISCORD_API_BASE_URL: &str = "https://discord.com/api/v10";
pub const DASHBOARD_URL: &str = "http://localhost:5173";

pub struct AppState {
    database: Database,
    api_host: String,
    webpage: String,
}

pub type AppStateArc = Arc<AppState>;

#[event(start)]
fn start() {
    // let fmt_layer = tracing_subscriber::fmt::layer()
    //     .json()
    //     .with_ansi(false)
    //     .with_timer(UtcTime::rfc_3339())
    //     .with_writer(MakeConsoleWriter);

    // let perf_layer = performance_layer().with_details_from_fields(Pretty::default());
    // tracing_subscriber::registry()
    //     .with(fmt_layer)
    //     .with(perf_layer)
    //     .init();
}

fn cors_layer(webpage: &str) -> CorsLayer {
    let webpage_header = HeaderValue::from_str(webpage).expect("Invalid URL for CORS");
    CorsLayer::new()
        .allow_origin(webpage_header)
        .allow_methods(vec![Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_credentials(true)
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

    let Ok(webpage) = env.var("DASHBOARD_URL").map(|s| s.to_string()) else {
        console_error!("Failed to get DASHBOARD_URL");
        return Ok(Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(Body::from("Internal Server Error"))
            .unwrap());
    };

    let Ok(api_host) = env.var("API_HOST").map(|s| s.to_string()) else {
        console_error!("Failed to get API_HOST");
        return Ok(Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(Body::from("Internal Server Error"))
            .unwrap());
    };

    let app_state = Arc::new(AppState {
        // Initialize your application state here
        database: Database::new(hyperdrive),
        webpage: webpage.clone(),
        api_host,
    });

    let mut app = Router::new()
        .nest("/api", api::router())
        .nest("/cdn", cdn::router())
        .route("/", get(root))
        .fallback(fallback)
        .layer(Extension(app_state))
        .layer(Extension(env))
        .layer(cors_layer(&webpage));

    Ok(app.call(req).await?)
}

async fn fallback() -> Response<Body> {
    Response::new(Body::from("Not Found"))
}

pub async fn root() -> &'static str {
    "Hello Axum!"
}
