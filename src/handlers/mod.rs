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

fn parse_format(format_str: &str) -> Option<(image::ImageFormat, &'static str)> {
    match format_str.to_lowercase().as_str() {
        "jpg" | "jpeg" => Some((image::ImageFormat::Jpeg, "image/jpeg")),
        "png" => Some((image::ImageFormat::Png, "image/png")),
        "webp" => Some((image::ImageFormat::WebP, "image/webp")),
        "avif" => Some((image::ImageFormat::Avif, "image/avif")),
        _ => None,
    }
}

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
    
    // Check if we need to process the image (resize or format conversion)
    let needs_processing = content_type.starts_with("image/") && 
        (params.contains_key("size") || params.contains_key("format"));
    
    if needs_processing {
        let bytes = response.bytes().await.map_err(|_| StatusCode::BAD_GATEWAY)?;
        
        // Load the image
        let mut img = image::load_from_memory(&bytes).map_err(|_| StatusCode::UNPROCESSABLE_ENTITY)?;
        
        // Apply resize if requested
        if let Some(size_str) = params.get("size") {
            if let Ok(width) = size_str.parse::<u32>() {
                let width = width.min(2048);
                info!("Resizing image to width: {}", width);
                img = img.resize(width, u32::MAX, image::imageops::FilterType::Lanczos3);
            }
        }
        
        // Determine output format
        let (output_format, output_content_type) = if let Some(format_str) = params.get("format") {
            if let Some((format, content_type)) = parse_format(format_str) {
                info!("Converting image to format: {}", format_str);
                (format, content_type)
            } else {
                return Err(StatusCode::BAD_REQUEST);
            }
        } else {
            // Use original format if no format conversion requested
            let original_format = image::guess_format(&bytes).map_err(|_| StatusCode::UNSUPPORTED_MEDIA_TYPE)?;
            (original_format, content_type.as_str())
        };
        
        // Encode the processed image
        let mut buffer = Vec::new();
        
        // Handle AVIF with speed optimization
        if matches!(output_format, image::ImageFormat::Avif) {
            use image::codecs::avif::AvifEncoder;
            let encoder = AvifEncoder::new_with_speed_quality(&mut buffer, 8, 60); // Fast speed, good quality
            img.write_with_encoder(encoder).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        } else {
            let mut cursor = Cursor::new(&mut buffer);
            img.write_to(&mut cursor, output_format).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        }
        
        return Ok(Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, output_content_type)
            .header(header::CONTENT_LENGTH, buffer.len())
            .body(Body::from(buffer))
            .unwrap());
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