use axum::{
    extract::{Path, Extension},
    http::{header, StatusCode},
    response::Response,
    body::Body,
};
use reqwest::Client;
use tracing::info;

pub async fn proxy_request(
    Extension(upstream_base_url): Extension<String>,
    Path(path): Path<String>,
) -> Result<Response<Body>, StatusCode> {
    info!("Proxying request: /{}", path);
    
    let client = Client::new();
    let upstream_url = format!("{}/{}", upstream_base_url, path);
    
    let response = client
        .get(&upstream_url)
        .send()
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;
    
    if !response.status().is_success() {
        return Err(StatusCode::from_u16(response.status().as_u16()).unwrap_or(StatusCode::BAD_GATEWAY));
    }
    
    let mut builder = Response::builder().status(StatusCode::OK);
    
    if let Some(content_type) = response.headers().get(header::CONTENT_TYPE) {
        builder = builder.header(header::CONTENT_TYPE, content_type);
    }
    
    if let Some(content_length) = response.headers().get(header::CONTENT_LENGTH) {
        builder = builder.header(header::CONTENT_LENGTH, content_length);
    }
    
    let stream = response.bytes_stream();
    let body = Body::wrap_stream(stream);
    
    Ok(builder.body(body).unwrap())
}