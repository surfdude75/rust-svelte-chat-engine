use std::{
    str::FromStr,
    sync::{Arc, Weak},
};

use futures_util::StreamExt;
use serde_json::{json, Value};
use warp::{reject::Rejection, reply::Reply, Filter};

use super::chat::{client::WebSocketClient, ChatManager};

pub fn websocket_filter(
    chat_manager: Arc<ChatManager>,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    let chat_manager_filter = warp::any().map(move || Arc::clone(&chat_manager));
    warp::path("ws")
        .and(warp::ws())
        .and(chat_manager_filter.clone())
        .map(|ws: warp::ws::Ws, chat_manager: Arc<ChatManager>| {
            ws.on_upgrade(|ws| async move {
                let (sender, mut websocket_listener) = ws.split();
                let websocket_client = chat_manager.create_client(sender).await;
                let client_id = websocket_client.get_id();
                websocket_client
                    .send(json!({
                        "type": "EVENT",
                        "event": {
                            "type": "CLIENT_JOIN",
                            "client_id": client_id.to_string()
                        }
                    }))
                    .await;
                let websocket_client: Weak<WebSocketClient> = Arc::downgrade(&websocket_client);
                while let Some(message) = websocket_listener.next().await {
                    match message {
                        Ok(message) => match message.to_str() {
                            Ok(message) => match websocket_client.upgrade() {
                                Some(websocket_client) => match Value::from_str(message) {
                                    Ok(value) => websocket_client.exec(&value).await,
                                    Err(e) => {
                                        println!("{} error parsing JSON {}", client_id, e)
                                    }
                                },
                                None => {
                                    println!("{} client was dropped", client_id);
                                    break;
                                }
                            },
                            Err(e) => {
                                println!("{} error getting string from socket {:?}", client_id, e)
                            }
                        },
                        Err(e) => println!("{} error reading socket {:?}", client_id, e),
                    }
                }
                chat_manager.remove_client(client_id).await;
            })
        })
}
