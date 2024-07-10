use axum::{
    routing::{get, post, put},
    Router,
};
use handlers::{
    authorise_check, authorization_middleware, create_transaction_handler, fallback_handler,
    list_transaction_handler, login_handler, modify_user_handler, register_handler,
    user_balance_handler,
};
mod errors;
mod handlers;
mod service;
mod utils;

pub mod config;
pub fn trnx_service() -> Router {
    //TODO: implement shared state among handlers eg  auth details
    Router::new()
        .route("/register", post(register_handler))
        .route("/login", post(login_handler))
        .route(
            "/user",
            put(modify_user_handler).layer(axum::middleware::from_fn(authorization_middleware)),
        )
        .route(
            "/authorise",
            post(authorise_check).layer(axum::middleware::from_fn(authorization_middleware)),
        )
        .route(
            "/balance",
            get(user_balance_handler).layer(axum::middleware::from_fn(authorization_middleware)),
        )
        .route(
            "/transaction",
            post(create_transaction_handler)
                .layer(axum::middleware::from_fn(authorization_middleware)),
        )
        .route(
            "/transaction",
            get(list_transaction_handler)
                .layer(axum::middleware::from_fn(authorization_middleware)),
        )
        .fallback(fallback_handler)
}
