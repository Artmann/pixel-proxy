use axum::{
    extract::{Path, Extension, Query},
    http::{header, StatusCode},
    response::Response,
    body::Body,
};
use reqwest::Client;
use tracing::info;
use image;
use std::collections::HashMap;
use std::io::Cursor;

pub async fn proxy_request(
    Extension(upstream_base_url): Extension<String>,
    Path(path): Path<String>,
    Query(params): Query<HashMap<String, String>>,
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
    
    let content_type = response
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|h| h.to_str().ok())
        .unwrap_or("")
        .to_string();
    
    // Check if we need to resize and if it's an image
    if let Some(size_str) = params.get("size") {
        if content_type.starts_with("image/") {
            if let Ok(width) = size_str.parse::<u32>() {
                let width = width.min(2048);
                info!("Resizing image to width: {}", width);
                
                let bytes = response.bytes().await.map_err(|_| StatusCode::BAD_GATEWAY)?;
                
                // Detect image format
                let format = image::guess_format(&bytes).map_err(|_| StatusCode::UNSUPPORTED_MEDIA_TYPE)?;
                
                // Load and resize image
                let img = image::load_from_memory(&bytes).map_err(|_| StatusCode::UNPROCESSABLE_ENTITY)?;
                let resized = img.resize(width, u32::MAX, image::imageops::FilterType::Lanczos3);
                
                // Encode resized image
                let mut buffer = Vec::new();
                let mut cursor = Cursor::new(&mut buffer);
                resized.write_to(&mut cursor, format).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
                
                return Ok(Response::builder()
                    .status(StatusCode::OK)
                    .header(header::CONTENT_TYPE, &content_type)
                    .header(header::CONTENT_LENGTH, buffer.len())
                    .body(Body::from(buffer))
                    .unwrap());
            }
        }
    }
    
    // Fallback: stream original image without resizing
    let mut builder = Response::builder().status(StatusCode::OK);
    
    if !content_type.is_empty() {
        builder = builder.header(header::CONTENT_TYPE, content_type);
    }
    
    if let Some(content_length) = response.headers().get(header::CONTENT_LENGTH) {
        builder = builder.header(header::CONTENT_LENGTH, content_length);
    }
    
    let stream = response.bytes_stream();
    let body = Body::wrap_stream(stream);
    
    Ok(builder.body(body).unwrap())
}