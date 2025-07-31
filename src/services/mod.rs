use worker::{console_error, Env};

pub mod auth;
pub mod cookie;
pub mod database;
pub mod user;

pub fn get_discord_env(env: &Env) -> Result<(String, String), String> {
    let Ok(client_id) = env.var("DISCORD_CLIENT_ID").map(|s| s.to_string()) else {
        console_error!("DISCORD_CLIENT_ID not set");
        return Err("DISCORD_CLIENT_ID not set".into());
    };
    let Ok(client_secret) = env.secret("DISCORD_CLIENT_SECRET").map(|s| s.to_string()) else {
        console_error!("DISCORD_CLIENT_SECRET not set");
        return Err("DISCORD_CLIENT_SECRET not set".into());
    };
    Ok((client_id, client_secret))
}
