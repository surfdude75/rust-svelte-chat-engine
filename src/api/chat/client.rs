use std::{
    collections::HashSet,
    str::FromStr,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Weak,
    },
};

use futures_util::{stream::SplitSink, SinkExt};
use serde_json::{json, Value};
use tokio::{
    spawn,
    sync::{Mutex, RwLock},
};
use uuid::Uuid;
use warp::filters::ws::{Message, WebSocket};

use crate::api::chat::room::WebSocketRoom;

use super::engine::ChatEngine;

pub struct WebSocketClient {
    manager: Weak<ChatEngine>,
    websocket_sender: Mutex<SplitSink<WebSocket, Message>>,
    rooms: RwLock<HashSet<Uuid>>,
    id: Uuid,
    subscribe_rooms: AtomicBool,
    subscribe_rooms_is_running: AtomicBool,
}

impl WebSocketClient {
    pub(super) fn new(
        manager: Weak<ChatEngine>,
        websocket_sender: SplitSink<WebSocket, Message>,
    ) -> Self {
        let id = Uuid::new_v4();
        let websocket_sender = Mutex::new(websocket_sender);
        let rooms = RwLock::new(HashSet::new());
        Self {
            manager,
            websocket_sender,
            rooms,
            id,
            subscribe_rooms: AtomicBool::new(false),
            subscribe_rooms_is_running: AtomicBool::new(false),
        }
    }

    pub async fn subscribe_rooms(&self) {
        self.subscribe_rooms.store(true, Ordering::Relaxed);
        if self.subscribe_rooms_is_running.load(Ordering::Relaxed) {
            return;
        }
        if let Some(manager) = self.manager.upgrade() {
            if let Some(client) = manager.get_client(&self.id).await {
                let client_id = *client.get_id();
                let client = Arc::downgrade(&client);
                let mut listener = manager.get_listener().await;
                spawn(async move {
                    while let Ok(value) = listener.recv().await {
                        if let Some(client) = client.upgrade() {
                            client.send(value).await;
                            if !client.subscribe_rooms.load(Ordering::Relaxed) {
                                break;
                            }
                        } else {
                            break;
                        }
                    }
                    println!("{} room subscription to client task exit", client_id);
                });
            }
        }
    }
    pub async fn unsubscribe_rooms(&self) {
        self.subscribe_rooms.store(false, Ordering::Relaxed);
        self.subscribe_rooms_is_running
            .store(false, Ordering::Relaxed);
    }

    pub async fn join_room(&self, room_id: &Uuid) {
        if let Some(manager) = self.manager.upgrade() {
            if let Some(room) = manager.get_room(room_id).await {
                if let Some(client) = manager.get_client(&self.id).await {
                    let client_id = *client.get_id();
                    println!("client {} join {}", client_id, room_id);
                    room.client_add(&client_id).await;
                    {
                        self.rooms.write().await.insert(*room_id);
                    }
                    let mut listener = room.get_listener().await;
                    {
                        let room = Arc::downgrade(&room);
                        let client = Arc::downgrade(&client);
                        spawn(async move {
                            while let Ok(value) = listener.recv().await {
                                println!("{} room to client exec signal {}", client_id, value);
                                if value["type"] == "EVENT"
                                    && value["event"]["type"] == "ROOM_EXIT"
                                    && value["event"]["client_id"] == client_id.to_string()
                                {
                                    break;
                                }
                                if let Some(room) = room.upgrade() {
                                    if room.has_client(&client_id).await {
                                        match client.upgrade() {
                                            Some(websocket_client) => {
                                                println!("{} room sending {}", client_id, value);
                                                websocket_client.send(value).await;
                                            }
                                            None => break,
                                        }
                                    } else {
                                        break;
                                    }
                                }
                            }
                            println!("{} room to client task exit", client_id);
                        });
                    }
                    println!("DEBUG {} client joined room {}", client_id, room_id);
                    room.exec(
                        &json!({
                            "action": "BROADCAST",
                            "data": {
                                "type": "EVENT",
                                "event": {
                                    "type": "ROOM_JOIN",
                                    "room_id": room_id.to_string(),
                                    "client_id": client_id.to_string()
                            } }
                        }),
                        Some(&client_id),
                    )
                    .await;
                }
            }
        }
    }
    pub fn get_id(&self) -> &Uuid {
        &self.id
    }
    pub async fn send(&self, value: Value) {
        let mut websocket_sender = self.websocket_sender.lock().await;
        websocket_sender
            .send(Message::text(value.to_string()))
            .await
            .unwrap_or_else(|e| {
                println!("{} failed to send message {} {}", &self.get_id(), e, value)
            })
    }
    pub async fn exec(&self, value: &Value) {
        if let Some(manager) = self.manager.upgrade() {
            println!("{} client broadcasting {}", self.get_id(), value);
            if value["action"] == "ROOM_CREATE" {
                let room = WebSocketRoom::create_room(&manager, &self.id).await;
                self.join_room(room.get_id()).await;
            } else if value["action"] == "ROOMS_SUBSCRIBE" {
                self.subscribe_rooms().await;
                self.send_rooms_list().await;
            } else if value["action"] == "ROOMS_UNSUBSCRIBE" {
                self.unsubscribe_rooms().await;
            } else if value["action"] == "ROOMS_LIST" {
                self.send_rooms_list().await;
            } else if value["action"] == "ROOM_JOIN" {
                let room_id = value["room_id"]
                    .as_str()
                    .and_then(|s| Uuid::from_str(s).ok());
                if let Some(room_id) = room_id {
                    println!("ROOM_JOIN {}", room_id);
                    self.join_room(&room_id).await;
                }
            } else if value["action"] == "ROOM_CLIENTS_LIST" {
                let room_id = value["room_id"]
                    .as_str()
                    .and_then(|s| Uuid::from_str(s).ok());
                if let Some(room_id) = room_id {
                    self.send_room_clients_list(&room_id).await;
                }
            } else if value["target"]["type"] == "ROOM" {
                let room_id = value["target"]["id"]
                    .as_str()
                    .and_then(|s| Uuid::from_str(s).ok());
                if let Some(room_id) = room_id {
                    if let Some(room) = manager.get_room(&room_id).await {
                        room.exec(value, Some(self.get_id())).await;
                    }
                }
            }
        }
    }

    async fn send_rooms_list(&self) {
        if let Some(manager) = self.manager.upgrade() {
            let rooms_list: Vec<String> = manager
                .get_rooms_list()
                .await
                .iter()
                .map(|id| id.to_string())
                .collect();

            self.send(json!({
                "type": "EVENT",
                "event": {
                    "type": "ROOMS_LIST",
                    "rooms": rooms_list
                }
            }))
            .await;
        }
    }

    async fn send_room_clients_list(&self, room_id: &Uuid) {
        if let Some(manager) = self.manager.upgrade() {
            if let Some(room) = manager.get_room(room_id).await {
                let client_list: Vec<String> = room
                    .get_clients_list()
                    .await
                    .iter()
                    .map(|id| id.to_string())
                    .collect();
                self.send(json!({
                    "type": "EVENT",
                    "event": {
                        "type": "ROOM_CLIENTS_LIST",
                        "clients": client_list,
                        "room_id": room_id.to_string()
                    }
                }))
                .await;
            }
        }
    }
    pub(super) async fn get_client_rooms(&self) -> Vec<Uuid> {
        self.rooms.read().await.iter().copied().collect()
    }
}

impl Drop for WebSocketClient {
    fn drop(&mut self) {
        println!("{} client dropped", self.get_id());
    }
}
