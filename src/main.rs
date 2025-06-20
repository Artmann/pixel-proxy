use pixel_proxy::create_app;
use std::env;
use tokio::net::TcpListener;
use tracing_subscriber;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_target(false)
        .with_max_level(tracing::Level::INFO)
        .compact()
        .init();
    let upstream_base_url = env::var("UPSTREAM_BASE_URL")
        .unwrap_or_else(|_| "https://gustavskitchen.se".to_string());
    
    let port = env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse::<u16>()
        .unwrap_or(3000);
    
    println!("🚀 Starting Pixel Proxy server");
    println!("Upstream server: {}", upstream_base_url);
    
    let app = create_app(upstream_base_url);

    let addr = format!("0.0.0.0:{}", port);
    println!("Server running on http://127.0.0.1");
    
    let listener = TcpListener::bind(&addr).await.unwrap();
    axum::Server::from_tcp(listener.into_std().unwrap())
        .unwrap()
        .serve(app.into_make_service())
        .await
        .unwrap();
}
