use std::{convert::Infallible, sync::Arc};

use chat_engine::api::{chat::ChatManager, websocket::websocket_filter};
use rust_embed::RustEmbed;
use tokio::spawn;
use warp::{
    http::StatusCode,
    reject::Rejection,
    reply::{with_status, Reply},
    Filter,
};

#[derive(RustEmbed)]
#[folder = "web/dist"]
struct Static;

async fn handle_rejection(error: Rejection) -> Result<impl Reply, Infallible> {
    if error.is_not_found() {
        Ok(with_status("NOT FOUND", StatusCode::NOT_FOUND))
    } else {
        Ok(with_status(
            "SERVER ERROR",
            StatusCode::INTERNAL_SERVER_ERROR,
        ))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<(dyn std::error::Error + 'static)>> {
    let websocket_manager = Arc::new(ChatManager::default());

    let websocket_api = websocket_filter(Arc::clone(&websocket_manager));

    let static_content = warp::any()
        .and(warp::get())
        .and(warp_embed::embed(&Static))
        .boxed();

    let routes = websocket_api.or(static_content);

    let routes = routes.recover(handle_rejection);
    let routes = routes.with(warp::cors());

    let http_server_handle = spawn(async move {
        warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
    });

    http_server_handle.await?;
    Ok(())
}
