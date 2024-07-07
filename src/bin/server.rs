use axum::{
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use tracing_subscriber::EnvFilter;
use transaction_service::config;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .pretty()
        .init();
    let server_addr = SocketAddr::from(([127, 0, 0, 1], 3042));

    let trnx_service = Router::new()
        .route("/", get(|| async { "Hello, World!" })) //TODO: implement shared state among handlers eg
        //like sharing db connection or auth details
        .fallback(fallback_handler);

    println!("Server started on {}", server_addr);
    let listener = tokio::net::TcpListener::bind(server_addr).await.unwrap();
    axum::serve(listener, trnx_service).await.unwrap();
}
