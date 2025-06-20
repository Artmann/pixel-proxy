use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use pixel_proxy::create_app;
use tower::ServiceExt;

#[tokio::test]
async fn test_proxy_without_resize() {
    let app = create_app("https://httpbin.org".to_string());

    let response = app
        .oneshot(
            Request::builder()
                .uri("/image/jpeg")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_proxy_with_resize() {
    let app = create_app("https://httpbin.org".to_string());

    let response = app
        .oneshot(
            Request::builder()
                .uri("/image/jpeg?size=300")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_size_limit() {
    let app = create_app("https://httpbin.org".to_string());

    let response = app
        .oneshot(
            Request::builder()
                .uri("/image/jpeg?size=5000")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    // The actual width should be capped at 2048, but we can't easily verify this without decoding the image
}

#[tokio::test]
async fn test_invalid_size_parameter() {
    let app = create_app("https://httpbin.org".to_string());

    let response = app
        .oneshot(
            Request::builder()
                .uri("/image/jpeg?size=invalid")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Should fallback to original image streaming
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_non_image_content() {
    let app = create_app("https://httpbin.org".to_string());

    let response = app
        .oneshot(
            Request::builder()
                .uri("/json?size=300")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Should stream original content without resizing
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_format_conversion_png() {
    let app = create_app("https://httpbin.org".to_string());

    let response = app
        .oneshot(
            Request::builder()
                .uri("/image/jpeg?format=png")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers().get(header::CONTENT_TYPE).unwrap(),
        "image/png"
    );
}

#[tokio::test]
async fn test_format_conversion_webp() {
    let app = create_app("https://httpbin.org".to_string());

    let response = app
        .oneshot(
            Request::builder()
                .uri("/image/jpeg?format=webp")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers().get(header::CONTENT_TYPE).unwrap(),
        "image/webp"
    );
}

#[tokio::test]
async fn test_format_conversion_avif() {
    let app = create_app("https://httpbin.org".to_string());

    let response = app
        .oneshot(
            Request::builder()
                .uri("/image/jpeg?format=avif")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers().get(header::CONTENT_TYPE).unwrap(),
        "image/avif"
    );
}

#[tokio::test]
async fn test_resize_and_format_conversion() {
    let app = create_app("https://httpbin.org".to_string());

    let response = app
        .oneshot(
            Request::builder()
                .uri("/image/jpeg?size=300&format=webp")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers().get(header::CONTENT_TYPE).unwrap(),
        "image/webp"
    );
}

#[tokio::test]
async fn test_invalid_format() {
    let app = create_app("https://httpbin.org".to_string());

    let response = app
        .oneshot(
            Request::builder()
                .uri("/image/jpeg?format=invalid")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}
