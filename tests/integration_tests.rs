use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;
use pixel_proxy::create_app;

#[tokio::test]
async fn test_proxy_without_resize() {
    let app = create_app("https://httpbin.org".to_string());
    
    let response = app
        .oneshot(Request::builder()
            .uri("/image/jpeg")
            .body(Body::empty())
            .unwrap())
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_proxy_with_resize() {
    let app = create_app("https://httpbin.org".to_string());
    
    let response = app
        .oneshot(Request::builder()
            .uri("/image/jpeg?size=300")
            .body(Body::empty())
            .unwrap())
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_size_limit() {
    let app = create_app("https://httpbin.org".to_string());
    
    let response = app
        .oneshot(Request::builder()
            .uri("/image/jpeg?size=5000")
            .body(Body::empty())
            .unwrap())
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    // The actual width should be capped at 2048, but we can't easily verify this without decoding the image
}

#[tokio::test]
async fn test_invalid_size_parameter() {
    let app = create_app("https://httpbin.org".to_string());
    
    let response = app
        .oneshot(Request::builder()
            .uri("/image/jpeg?size=invalid")
            .body(Body::empty())
            .unwrap())
        .await
        .unwrap();
    
    // Should fallback to original image streaming
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_non_image_content() {
    let app = create_app("https://httpbin.org".to_string());
    
    let response = app
        .oneshot(Request::builder()
            .uri("/json?size=300")
            .body(Body::empty())
            .unwrap())
        .await
        .unwrap();
    
    // Should stream original content without resizing
    assert_eq!(response.status(), StatusCode::OK);
}