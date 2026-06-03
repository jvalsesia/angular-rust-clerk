use axum::{
    body::Body,
    http::{
        Request, StatusCode,
        header::{ACCESS_CONTROL_ALLOW_ORIGIN, ACCESS_CONTROL_REQUEST_METHOD, ORIGIN},
    },
};
use backend::{config::Config, create_app};
use tower::ServiceExt;

#[tokio::test]
async fn test_cors_preflight_success() {
    let config = Config {
        port: 3000,
        allowed_origins: vec!["http://localhost:4200".to_string()],
        clerk_jwks_url: "https://api.clerk.com/v1/jwks".to_string(),
        clerk_issuer: "https://gentle-ophaph-98.clerk.accounts.dev".to_string(),
        database_url: "postgres://postgres:postgres@localhost:5432/chat_db".to_string(),
        litellm_url: "http://localhost:4000".to_string(),
        litellm_api_key: "".to_string(),
    };
    let app = create_app(&config, None);

    let req = Request::builder()
        .method("OPTIONS")
        .uri("/api/health")
        .header(ORIGIN, "http://localhost:4200")
        .header(ACCESS_CONTROL_REQUEST_METHOD, "GET")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(req).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let cors_header = response.headers().get(ACCESS_CONTROL_ALLOW_ORIGIN);
    assert!(cors_header.is_some());
    assert_eq!(cors_header.unwrap(), "http://localhost:4200");
}

#[tokio::test]
async fn test_cors_invalid_origin() {
    let config = Config {
        port: 3000,
        allowed_origins: vec!["http://localhost:4200".to_string()],
        clerk_jwks_url: "https://api.clerk.com/v1/jwks".to_string(),
        clerk_issuer: "https://gentle-ophaph-98.clerk.accounts.dev".to_string(),
        database_url: "postgres://postgres:postgres@localhost:5432/chat_db".to_string(),
        litellm_url: "http://localhost:4000".to_string(),
        litellm_api_key: "".to_string(),
    };
    let app = create_app(&config, None);

    let req = Request::builder()
        .method("OPTIONS")
        .uri("/api/health")
        .header(ORIGIN, "http://disallowed.com")
        .header(ACCESS_CONTROL_REQUEST_METHOD, "GET")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(req).await.unwrap();

    println!("Response headers: {:?}", response.headers());

    let cors_header = response.headers().get(ACCESS_CONTROL_ALLOW_ORIGIN);
    assert!(cors_header.is_none());
}
