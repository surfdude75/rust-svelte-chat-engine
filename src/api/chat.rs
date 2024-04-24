use std::sync::Arc;

use futures_util::{future::join_all, stream::SplitSink};
use uuid::Uuid;
use warp::filters::ws::{Message, WebSocket};

use self::{client::WebSocketClient, engine::ChatEngine, room::WebSocketRoom};

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

    pub async fn create_room(&self, clients: Vec<&Uuid>) -> Option<Arc<WebSocketRoom>> {
        let mut clients: Vec<Arc<WebSocketClient>> = join_all(
            clients
                .iter()
                .map(|client_id| async { self.engine.get_client(client_id).await }),
        )
        .await
        .into_iter()
        .flatten()
        .collect();

        if let Some(creator) = clients.pop() {
            let room = WebSocketRoom::create_room(&self.engine, creator.get_id()).await;
            for client in clients {
                client.join_room(room.get_id()).await;
            }
            return Some(room);
        }
        None
    }
}

impl Default for ChatManager {
    fn default() -> Self {
        ChatManager {
            engine: Arc::new(ChatEngine::default()),
        }
    }
}
