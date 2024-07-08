use axum::{
    routing::{get, post},
    Router,
};
use handlers::{fallback_handler, login_register, register_handler};
mod errors;
mod handlers;
mod utils;

pub mod config;
pub fn trnx_service() -> Router {
    //TODO: implement shared state among handlers eg  auth details
    Router::new()
        .route("/register", post(register_handler))
        .route("/login", post(login_register))
        .fallback(fallback_handler)
}
