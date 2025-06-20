pub mod handlers;
pub mod services;

use axum::{
    routing::get,
    Router,
    Extension,
};
use tower_http::trace::TraceLayer;

pub fn create_app(upstream_base_url: String) -> Router {
    Router::new()
        .route("/*path", get(handlers::proxy_request))
        .layer(Extension(upstream_base_url))
        .layer(TraceLayer::new_for_http())
}