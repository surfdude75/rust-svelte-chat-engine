use std::{collections::HashMap, sync::Arc};

use futures_util::stream::SplitSink;
use serde_json::{json, Value};
use tokio::sync::{
    broadcast::{self, Receiver, Sender},
    Mutex, RwLock,
};
use uuid::Uuid;
use warp::filters::ws::{Message, WebSocket};

use super::{client::WebSocketClient, room::WebSocketRoom};

pub(super) struct ChatEngine {
    rooms: RwLock<HashMap<Uuid, Arc<WebSocketRoom>>>,
    clients: RwLock<HashMap<Uuid, Arc<WebSocketClient>>>,
    sender: Mutex<Sender<Value>>,
}

impl Default for ChatEngine {
    fn default() -> Self {
        let (sender, _) = broadcast::channel(1);
        Self {
            clients: RwLock::new(HashMap::new()),
            rooms: RwLock::new(HashMap::new()),
            sender: Mutex::new(sender),
        }
    }
}

impl ChatEngine {
    pub(super) async fn get_room(&self, room_id: &Uuid) -> Option<Arc<WebSocketRoom>> {
        self.rooms.read().await.get(room_id).cloned()
    }
    pub(super) async fn get_client(&self, client_id: &Uuid) -> Option<Arc<WebSocketClient>> {
        self.clients.read().await.get(client_id).cloned()
    }

    pub(super) async fn create_client(
        engine: &Arc<ChatEngine>,
        websocket_sender: SplitSink<WebSocket, Message>,
    ) -> Arc<WebSocketClient> {
        let mut clients = engine.clients.write().await;
        let engine = Arc::downgrade(engine);
        let client = Arc::new(WebSocketClient::new(engine, websocket_sender));
        clients.insert(*client.get_id(), Arc::clone(&client));
        println!("{} websocket engine created client", client.get_id());
        client
    }
    pub(super) async fn remove_client(&self, client_id: &Uuid) {
        if let Some(client) = self.get_client(client_id).await {
            let mut clients = self.clients.write().await;
            let rooms = client.get_client_rooms().await;
            clients.remove(client_id);
            for room_id in rooms {
                if let Some(room) = self.get_room(&room_id).await {
                    room.remove_client(client_id).await;
                }
            }
            println!("{} websocket engine removed client", client_id);
        }
    }
    pub(super) async fn get_listener(&self) -> Receiver<Value> {
        self.sender.lock().await.subscribe()
    }
    pub(super) async fn get_rooms_list(&self) -> Vec<Uuid> {
        self.rooms.read().await.keys().cloned().collect()
    }
    pub(super) async fn room_add(&self, room: &Arc<WebSocketRoom>) {
        {
            let mut rooms = self.rooms.write().await;
            rooms.insert(*room.get_id(), Arc::clone(room));
        }
        let message = json!({
            "type": "EVENT",
            "event": {
                "type": "ROOM_CREATION",
                "room_id": room.get_id().to_string(),
                "creator_id": room.get_creator().to_string()
            }
        });
        //TODO: Avoid lof error if no one is listening
        self.sender.lock().await.send(message).unwrap_or_else(|e| {
            println!("{} engine room add broadcast error {}", room.get_id(), e);
            0
        });
    }
    pub(super) async fn room_remove(&self, room_id: &Uuid) {
        {
            let mut rooms = self.rooms.write().await;
            rooms.remove(room_id);
        }
        let message = json!({
            "type": "EVENT",
            "event": {
                "type": "ROOM_REMOVAL",
                "room_id": room_id.to_string()
            }
        });
        self.sender.lock().await.send(message).unwrap_or_else(|e| {
            println!("{} engine room remove broadcast error {}", room_id, e);
            0
        });
    }
}
