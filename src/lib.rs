use axum::{
    extract::{rejection::JsonRejection, Json},
    routing::{get, post},
    Router,
};
mod handlers;

pub mod config;
pub fn trnx_service() {
    //TODO: implement shared state among handlers eg like sharing db connection or auth details
    let trnx_service = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .fallback(handlers::fallback_handler);
}
