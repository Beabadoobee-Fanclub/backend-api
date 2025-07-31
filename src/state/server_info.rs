use std::sync::Arc;

use reqwest::StatusCode;
use tracing::error;
use worker::{console_error, Env, Result};

#[derive(Debug, Clone)]
pub struct ServerInfo {
    api_host: String,
    webpage: String,
}

pub type ServerInfoArc = Arc<ServerInfo>;

impl ServerInfo {
    pub fn new(env: &Env) -> Result<Arc<Self>> {
        let api_host = env.var("API_HOST").map(|s| s.to_string())?;
        let webpage = env.var("DASHBOARD_URL").map(|s| s.to_string())?;
        Ok(Arc::new(Self { api_host, webpage }))
    }

    pub fn api_host(&self) -> &str {
        &self.api_host
    }
    pub fn webpage(&self) -> &str {
        &self.webpage
    }
}
