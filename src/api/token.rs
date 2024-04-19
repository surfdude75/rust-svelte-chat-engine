use std::sync::Arc;

use serde_json::json;
use warp::{
    http::StatusCode,
    reject::{Reject, Rejection},
    reply::{with_status, Reply},
    Filter,
};

use crate::token_manager::TokenManager;

#[derive(Debug)]
pub enum TokenError {
    TokenNotFound,
    NoMoreTokens,
}

impl Reject for TokenError {}

pub fn create(
    token_manager: Arc<TokenManager>,
) -> impl Filter<Error = Rejection, Extract = (impl Reply,)> + Clone {
    let token_manager_filter = warp::any().map(move || Arc::clone(&token_manager));
    warp::get()
        .and(warp::path("token"))
        .and(token_manager_filter.clone())
        .and(warp::path("create"))
        .and(warp::path::end())
        .and_then(|token_manager: Arc<TokenManager>| async move {
            match token_manager.create().await {
                Ok(token) => Ok::<_, Rejection>(with_status(
                    warp::reply::json(&json!({"token": token})),
                    StatusCode::OK,
                )),
                Err(_) => Ok::<_, Rejection>(with_status(
                    warp::reply::json(&json!({"error":"NO MORE TOKENS"})),
                    StatusCode::SERVICE_UNAVAILABLE,
                )),
            }
        })
}

pub fn list(
    token_manager: Arc<TokenManager>,
) -> impl Filter<Error = Rejection, Extract = (impl Reply,)> + Clone {
    let token_manager_filter = warp::any().map(move || Arc::clone(&token_manager));
    warp::get()
        .and(warp::path("token"))
        .and(token_manager_filter.clone())
        .and(warp::path("list"))
        .and(warp::path::end())
        .and_then(|token_manager: Arc<TokenManager>| async move {
            let list = token_manager.list().await;
            let json = json!({ "tokens": list });
            Ok::<_, Rejection>(warp::reply::json(&json))
        })
}

pub fn connect(
    token_manager: Arc<TokenManager>,
) -> impl Filter<Error = Rejection, Extract = (impl Reply,)> + Clone {
    let token_manager_filter = warp::any().map(move || Arc::clone(&token_manager));
    warp::get()
        .and(warp::path("token"))
        .and(token_manager_filter.clone())
        .and(warp::path("connect"))
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and_then(
            |token_manager: Arc<TokenManager>, token: String| async move {
                match token_manager.connect(&token).await {
                    Ok(_) => Ok::<_, Rejection>(with_status(
                        warp::reply::json(&json!({"status":"OK"})),
                        StatusCode::OK,
                    )),
                    Err(_) => Ok::<_, Rejection>(with_status(
                        warp::reply::json(&json!({"error":"TOKEN NOT FOUND"})),
                        StatusCode::NOT_FOUND,
                    )),
                }
            },
        )
}
