use std::{
    collections::HashSet,
    sync::{Arc, Weak},
};

use serde_json::{json, Value};
use tokio::sync::{
    broadcast::{self, Receiver, Sender},
    Mutex, RwLock,
};
use uuid::Uuid;

use super::engine::ChatEngine;

pub struct WebSocketRoom {
    id: Uuid,
    clients: RwLock<HashSet<Uuid>>,
    sender: Mutex<Sender<Value>>,
    engine: Weak<ChatEngine>,
    creator: Uuid,
}

impl WebSocketRoom {
    fn new(engine: &Arc<ChatEngine>, creator: &Uuid) -> Self {
        let (sender, _) = broadcast::channel(10); //TODO: Should be 1? Handle Lagged in client loop
        let engine = Arc::downgrade(engine);
        Self {
            id: Uuid::new_v4(),
            clients: RwLock::new(HashSet::new()),
            sender: Mutex::new(sender),
            engine,
            creator: *creator,
        }
    }
    pub(super) async fn client_add(&self, client_id: &Uuid) {
        self.clients.write().await.insert(*client_id);
    }
    pub(super) async fn create_room(
        engine: &Arc<ChatEngine>,
        creator: &Uuid,
    ) -> Arc<WebSocketRoom> {
        let websocket_room = Arc::new(WebSocketRoom::new(engine, creator));
        {
            engine.room_add(&websocket_room).await;
        }
        websocket_room
    }
    pub fn get_creator(&self) -> &Uuid {
        &self.creator
    }
    pub fn get_id(&self) -> &Uuid {
        &self.id
    }
    pub async fn get_clients_list(&self) -> Vec<Uuid> {
        self.clients.read().await.iter().cloned().collect()
    }
    pub async fn get_listener(&self) -> Receiver<Value> {
        self.sender.lock().await.subscribe()
    }
    pub async fn has_client(&self, client_id: &Uuid) -> bool {
        self.clients.read().await.contains(client_id)
    }
    pub(super) async fn remove_client(&self, client_id: &Uuid) {
        {
            let mut clients = self.clients.write().await;
            clients.remove(client_id);
            if clients.is_empty() {
                if let Some(engine) = self.engine.upgrade() {
                    engine.room_remove(&self.id).await;
                }
            }
        }
        let value = json!({
            "type": "EVENT",
            "event": {
                "type": "ROOM_EXIT",
                "client_id": client_id.to_string(),
                "room_id": self.get_id().to_string()
            }
        });
        self.sender.lock().await.send(value).unwrap_or_else(|e| {
            println!("{} room remove client broadcast error {}", self.get_id(), e);
            0
        });
    }
    pub async fn exec(&self, value: &Value, sender: Option<&Uuid>) {
        if value["action"] == "BROADCAST" {
            let mut value = value["data"].clone();
            if let Some(sender) = sender {
                value["sender"] = json!(sender.to_string());
            }
            value["room"] = json!(self.get_id().to_string());
            self.sender.lock().await.send(value).unwrap_or_else(|e| {
                println!("{} room exec broadcast error {}", self.get_id(), e);
                0
            });
        } else if value["action"] == "ROOM_EXIT" {
            self.remove_client(sender.unwrap()).await;
        } else if value["action"] == "SUBSCRIBE_ROOMS" {
        }
    }
}

impl Drop for WebSocketRoom {
    fn drop(&mut self) {
        println!("{} room dropped", self.get_id());
    }
}
