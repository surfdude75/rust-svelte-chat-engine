use std::sync::Arc;

use futures_util::stream::SplitSink;
use warp::filters::ws::{Message, WebSocket};

use self::{client::WebSocketClient, engine::ChatEngine};

pub mod client;
mod engine;
pub mod room;

pub struct ChatManager {
    engine: Arc<ChatEngine>,
}
impl ChatManager {
    pub async fn create_client(
        &self,
        sender: SplitSink<WebSocket, Message>,
    ) -> Arc<WebSocketClient> {
        ChatEngine::create_client(&self.engine, sender).await
    }
    pub async fn remove_client(&self, client_id: &uuid::Uuid) {
        self.engine.remove_client(client_id).await;
    }
}

impl Default for ChatManager {
    fn default() -> Self {
        ChatManager {
            engine: Arc::new(ChatEngine::default()),
        }
    }
}
