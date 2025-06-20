use axum::{
    body::Body,
    extract::{Extension, Path, Query},
    http::{header, StatusCode},
    response::Response,
};
use image;
use reqwest::Client;
use std::collections::HashMap;
use std::io::Cursor;
use std::time::Instant;
use tracing::info;

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
    let start_time = Instant::now();
    info!("Proxying request: /{}", path);

    let client = Client::new();
    let upstream_url = format!("{}/{}", upstream_base_url, path);

    let response = client
        .get(&upstream_url)
        .send()
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;

    if !response.status().is_success() {
        return Err(
            StatusCode::from_u16(response.status().as_u16()).unwrap_or(StatusCode::BAD_GATEWAY)
        );
    }

    let content_type = response
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|h| h.to_str().ok())
        .unwrap_or("")
        .to_string();

    // Check if we need to process the image (resize or format conversion)
    let needs_processing = content_type.starts_with("image/")
        && (params.contains_key("size") || params.contains_key("format"));

    if needs_processing {
        let bytes = response
            .bytes()
            .await
            .map_err(|_| StatusCode::BAD_GATEWAY)?;

        // Load the image
        let mut img =
            image::load_from_memory(&bytes).map_err(|_| StatusCode::UNPROCESSABLE_ENTITY)?;

        // Apply resize if requested
        if let Some(size_str) = params.get("size") {
            if let Ok(width) = size_str.parse::<u32>() {
                let width = width.min(2048);
                info!("Resizing image to width: {}", width);
                img = img.resize(width, u32::MAX, image::imageops::FilterType::Lanczos3);
            }
        }

        // Get quality parameter (only if explicitly specified)
        let quality = params.get("quality")
            .and_then(|q| q.parse::<u8>().ok())
            .map(|q| q.clamp(10, 100));

        // Determine output format
        let (output_format, output_content_type) = if let Some(format_str) = params.get("format") {
            if let Some((format, content_type)) = parse_format(format_str) {
                if let Some(q) = quality {
                    info!("Converting image to format: {} (quality: {})", format_str, q);
                } else {
                    info!("Converting image to format: {}", format_str);
                }
                (format, content_type)
            } else {
                return Err(StatusCode::BAD_REQUEST);
            }
        } else {
            // Use original format if no format conversion requested
            let original_format =
                image::guess_format(&bytes).map_err(|_| StatusCode::UNSUPPORTED_MEDIA_TYPE)?;
            (original_format, content_type.as_str())
        };

        // Encode the processed image with quality settings
        let mut buffer = Vec::new();

        match output_format {
            image::ImageFormat::Avif => {
                use image::codecs::avif::AvifEncoder;
                let quality_val = quality.unwrap_or(60); // Default AVIF quality
                let encoder = AvifEncoder::new_with_speed_quality(&mut buffer, 8, quality_val);
                img.write_with_encoder(encoder)
                    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            }
            image::ImageFormat::Jpeg => {
                use image::codecs::jpeg::JpegEncoder;
                let quality_val = quality.unwrap_or(80); // Default JPEG quality
                let encoder = JpegEncoder::new_with_quality(&mut buffer, quality_val);
                img.write_with_encoder(encoder)
                    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            }
            _ => {
                let mut cursor = Cursor::new(&mut buffer);
                img.write_to(&mut cursor, output_format)
                    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            }
        }

        let duration = start_time.elapsed();
        info!(
            "Request /{} completed in {}ms (processed)",
            path,
            duration.as_millis()
        );

        return Ok(Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, output_content_type)
            .header(header::CONTENT_LENGTH, buffer.len())
            .header(header::CACHE_CONTROL, "public, max-age=31536000, immutable")
            .header(header::EXPIRES, "Thu, 31 Dec 2037 23:55:55 GMT")
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

    // Add cache headers for streamed images
    builder = builder
        .header(header::CACHE_CONTROL, "public, max-age=31536000, immutable")
        .header(header::EXPIRES, "Thu, 31 Dec 2037 23:55:55 GMT");

    let stream = response.bytes_stream();
    let body = Body::wrap_stream(stream);

    let duration = start_time.elapsed();
    info!(
        "Request /{} completed in {}ms (streamed)",
        path,
        duration.as_millis()
    );

    Ok(builder.body(body).unwrap())
}
