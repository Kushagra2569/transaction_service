use std::net::SocketAddr;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new("info"))
        .pretty()
        .init();
    let server_addr = SocketAddr::from(([127, 0, 0, 1], 3042));

    println!("Server started on {}", server_addr);
    let listener = tokio::net::TcpListener::bind(server_addr).await.unwrap();
    let server = axum::serve(listener, transaction_service::trnx_service());

    if let Err(err) = server.await {
        tracing::error!("server error: {:?}", err);
    }
}
