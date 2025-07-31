use reqwest::header::USER_AGENT;
use worker::{console_log, durable_object, Env, Request, Response, Result, State, WebSocketPair};

#[durable_object]
pub struct BotRoom {
    state: State,
    env: Env,
}

impl DurableObject for BotRoom {
    fn new(state: State, env: Env) -> Self {
        BotRoom { state, env }
    }

    async fn fetch(&self, req: Request) -> Result<Response> {
        match req.headers().get("Upgrade") {
            Ok(Some(value)) => {
                if value != "websocket" {
                    return Response::error("Expected WebSocket upgrade", 400);
                }
            }
            _ => return Response::error("Expected WebSocket upgrade", 400),
        };

        let Some(user_agent) = req.headers().get(USER_AGENT.as_str())? else {
            return Response::error("Missing User-Agent header", 400);
        };

        let ws = WebSocketPair::new()?;
        let client = ws.client;
        let server = ws.server;

        if user_agent == "DiscordBot" {
            // This is a bot
            console_log!("New bot connected");
            self.state.accept_websocket_with_tags(&server, &["bot"]);
        } else if user_agent.starts_with("DiscordGuild") {
            // This is a dashboard connection fr
            console_log!("New guild client connected");
            let split_user_agent: Vec<&str> = user_agent.split('/').collect();
            let guild_id_str = split_user_agent.get(1).unwrap_or(&"unknown");
            console_log!("Guild ID: {}", guild_id_str);
            self.state
                .accept_websocket_with_tags(&server, &["guild", guild_id_str]);

            if let Err(e) = self.send_to_bot(&format!("Guild {} connected to bot", guild_id_str)) {
                console_log!("Failed to send message to bot: {}", e);
            }
        }

        Response::from_websocket(client)
    }
    async fn websocket_message(
        &self,
        ws: worker::WebSocket,
        message: worker::WebSocketIncomingMessage,
    ) -> Result<()> {
        match message {
            worker::WebSocketIncomingMessage::String(text) => {
                console_log!("Received text message: {}", text);
                // Handle text message
            }
            worker::WebSocketIncomingMessage::Binary(bits) => {
                console_log!("Received binary message of length: {}", bits.len());
                // Handle binary message
            }
        }
        Ok(())
    }

    async fn websocket_close(
        &self,
        ws: worker::WebSocket,
        code: usize,
        reason: String,
        was_clean: bool,
    ) -> Result<()> {
        console_log!(
            "WebSocket closed with code: {}, reason: {}, was_clean: {}",
            code,
            reason,
            was_clean
        );

        // Handle WebSocket close event
        Ok(())
    }

    async fn websocket_error(&self, ws: worker::WebSocket, error: worker::Error) -> Result<()> {
        console_log!("WebSocket error: {}", error);
        // Handle WebSocket error
        Ok(())
    }
}

impl BotRoom {
    fn send_to_bot(&self, message: &str) -> Result<()> {
        let connections = self.state.get_websockets_with_tag("bot");
        for ws in connections.iter() {
            if let Err(e) = ws.send_with_str(message) {
                console_log!("Failed to send message to bot: {}", e);
            }
        }
        Ok(())
    }

    fn send_to_dashboard(&self, guild_id: &str, message: &str) -> Result<()> {
        let connections = self.state.get_websockets_with_tag(guild_id);
        for ws in connections.iter() {
            if let Err(e) = ws.send_with_str(message) {
                console_log!("Failed to send message to guild {}: {}", guild_id, e);
            }
        }
        Ok(())
    }

    fn send_to_all_dashboards(&self, message: &str) -> Result<()> {
        let connections = self.state.get_websockets_with_tag("guild");
        for ws in connections.iter() {
            if let Err(e) = ws.send_with_str(message) {
                console_log!("Failed to send message to guild: {}", e);
            }
        }
        Ok(())
    }
}
