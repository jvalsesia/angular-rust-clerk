use axum::{
    body::Body,
    http::{Request, StatusCode, header::AUTHORIZATION},
    routing::post,
    Json, Router,
};
use backend::{config::Config, create_app};
use jsonwebtoken::{encode, Algorithm, Header, EncodingKey};
use std::time::{SystemTime, UNIX_EPOCH};
use tower::ServiceExt;
use serde_json::json;

#[derive(serde::Serialize, serde::Deserialize)]
struct TestClaims {
    sub: String,
    exp: u64,
    iss: String,
}

const TEST_PRIVATE_KEY_PEM: &[u8] = b"-----BEGIN PRIVATE KEY-----\n\
MIIEvgIBADANBgkqhkiG9w0BAQEFAASCBKgwggSkAgEAAoIBAQCnNcsfPIcEi1xE\n\
1z9NorTdbaaTSaw26ARLMOpfOC/8XYeltxhzv6brGzPufu58ed3XutqAfEIyMqPL\n\
99BimzTmSoirAW3YYQJyBDW5G+dfk5v01QCohOYr6phdF3u5tqxLTGlVXhrlMkZ+\n\
ld8ebzNrepZcjn6oypduPBR4h7yz0xxEWGgYR5RS4XJNW5R5uwRXJSmzsYb5rpiT\n\
aFFQ8RI8Pt34zPDl58BTSy9V7htjFzDnWQZDKeVp7w2xLlPnptNgLihTER+pEh6j\n\
PLZldch6AG85tdPDtf4PZT85iGHXmlgUarVK8Oh+1iS12nBOpymzVpzWiznx3ySH\n\
wN9WLokXAgMBAAECggEABVcpVNGSPw8Tvtp1xKKDjqZFEY44UwvyWyka9eNDBmyW\n\
+7Aki/SH6G7BfVwOJ5Py2xmLh1qwozl31CNm7fH8F2DeYEEsQ3gcY7eGXxJTlPtk\n\
xJtAU/bkAmy8j2NDyd4vlDttX9MgyRzjyyuOk+0Zoin+VG10ASsbTrjaLzFSpxbz\n\
/WEQpPz6xfLNWo9KWJGXF8H71FF+qXdF7lUFjLZd4B8FqP6WVIgVGGwk7ATM58lv\n\
t58rpOFQkAGAkmYm1E5myxroZrV2bcCuLE2flnFmUJL8mpPp2PUL7pZunxuNiMfG\n\
Aaol2QJ9ZOz1Mw9NOvE0zpUbAbvslJ2dOdFp+GdgjQKBgQDRnR30ZonGrZyeBlzL\n\
fTNSf3rohlhxveFQC0qPWJ3OIWFHhjP5JGHpwpHAYABSsjcCzgPGKRt+0uIZLlEw\n\
fXTXeBqez6rpM877RPbJBjQjPQsCl9XpZNeLBhImMX+faHStzyDF15xgimylDqBo\n\
zsJE9M7RjyPwqpvCZN9fd7e4QwKBgQDMNnVg6hEpUf3q9gNiC4+LB6jWY+qgJ2fY\n\
h8FPsLT0bRs22Hx46dpFzI/uQgPEVqfw0XP/VhdwEe/Bixy/pC3lIxVKFcWCr/64\n\
B3vGL7PddbYHhVXTET1zjMvYnR7MIwmFUoou/OZeUizcqYzf0sMRFZ7l+x2KPJQk\n\
leO02S7YnQKBgBAGJM4UMKAhkYF7FwjvT0cVO74e1xAK5fiKhG6k5Ztmbdtb5Qk8\n\
wMdv+lhsflnUCeSK/zrc1Z9CW8p1Afvk+1OleNN/KJ+fOEl5IiyH7uBqwDa4iL/I\n\
17lnA2gsDIeRIqpO1UCKlQfETT3o+lZIyA0hcdYPTT4OrM2VjIXtzvulAoGBAKni\n\
RBOzpUMyqoHk1zuhUnDell6EEJPbNFC13uNkpaURfypJPoN4R9T5MGONF4Umcd+s\n\
30rzW9wnj8T67ZegBW4xmWxgYEcwEj8WOqnM1VzOp/fpvFQya2TNJGe3jf9Uxn7b\n\
A4nDagHdauTHSCKLOyvjSKUaGqD9dGBbMWspogchAoGBAM8rTN+TWv5DGfs5VJJG\n\
eg4db1Nuum6pqBKk9sfLWiMkgSd1Py3Cv3HI2kGnGfsy/A/LLEQADIxlGt799D8N\n\
+tiHpoZxs5oLPhkDbcIo0SvmHY+4r6NJ5Lm660NCvgDyb9W9RYW1EFwt82LLjgxy\n\
EwbuNV/ya+fjusyJd24Xke+J\n\
-----END PRIVATE KEY-----";

#[tokio::test]
async fn test_chat_endpoint_success() {
    // 1. Sprout local server to mock Clerk JWKS response
    let jwks_mock = Router::new().route("/jwks", axum::routing::get(|| async {
        Json(json!({
            "keys": [
                {
                    "kid": "test_kid",
                    "kty": "RSA",
                    "alg": "RS256",
                    "n": "pzXLHzyHBItcRNc_TaK03W2mk0msNugESzDqXzgv_F2HpbcYc7-m6xsz7n7ufHnd17ragHxCMjKjy_fQYps05kqIqwFt2GECcgQ1uRvnX5Ob9NUAqITmK-qYXRd7ubasS0xpVV4a5TJGfpXfHm8za3qWXI5-qMqXbjwUeIe8s9McRFhoGEeUUuFyTVuUebsEVyUps7GG-a6Yk2hRUPESPD7d-Mzw5efAU0svVe4bYxcw51kGQynlae8NsS5T56bTYC4oUxEfqRIeozy2ZXXIegBvObXTw7X-D2U_OYhh15pYFGq1SvDoftYktdpwTqcps1ac1os58d8kh8DfVi6JFw",
                    "e": "AQAB"
                }
            ]
        }))
    }));

    let jwks_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let jwks_addr = jwks_listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(jwks_listener, jwks_mock).await.unwrap();
    });

    // 2. Sprout local server to mock LiteLLM completions/embeddings
    let litellm_mock = Router::new()
        .route("/v1/embeddings", post(|| async {
            Json(json!({
                "data": [
                    {
                        "embedding": vec![0.1; 1536]
                    }
                ]
            }))
        }))
        .route("/v1/chat/completions", post(|| async {
            (
                [("content-type", "text/event-stream")],
                "data: {\"choices\": [{\"delta\": {\"content\": \"Hello\"}}]}\n\ndata: [DONE]\n\n"
            )
        }));

    let litellm_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let litellm_addr = litellm_listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(litellm_listener, litellm_mock).await.unwrap();
    });

    let mock_jwks_url = format!("http://{}/jwks", jwks_addr);
    let mock_litellm_url = format!("http://{}", litellm_addr);

    unsafe {
        std::env::set_var("CLERK_JWKS_URL", &mock_jwks_url);
        std::env::set_var("CLERK_ISSUER", "https://issuer.com");
    }

    // 3. Generate a valid token
    let claims = TestClaims {
        sub: "user_chat_123".to_string(),
        exp: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            + 3600,
        iss: "https://issuer.com".to_string(),
    };

    let mut header = Header::new(Algorithm::RS256);
    header.kid = Some("test_kid".to_string());

    let encoding_key = EncodingKey::from_rsa_pem(TEST_PRIVATE_KEY_PEM).unwrap();
    let token = encode(&header, &claims, &encoding_key).unwrap();

    // 4. Create App and send POST request to /api/chat
    let config = Config {
        port: 3000,
        allowed_origins: vec!["http://localhost:4200".to_string()],
        clerk_jwks_url: mock_jwks_url,
        clerk_issuer: "https://issuer.com".to_string(),
        database_url: "postgres://postgres:postgres@localhost:5432/chat_db".to_string(),
        litellm_url: mock_litellm_url,
        litellm_api_key: "sk-mock-key".to_string(),
    };
    let app = create_app(&config, None);

    let req = Request::builder()
        .method("POST")
        .uri("/api/chat")
        .header(AUTHORIZATION, format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .body(Body::from(json!({
            "session_id": null,
            "prompt": "Test Prompt",
            "model": "gpt-4o-mini"
        }).to_string()))
        .unwrap();

    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let content_type = response.headers().get("content-type").unwrap();
    assert_eq!(content_type, "text/event-stream");

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_str = String::from_utf8(body_bytes.to_vec()).unwrap();
    
    // Check that structured context and tokens were pushed
    println!("BODY: {}", body_str);
    assert!(body_str.contains("event: context"));
    assert!(body_str.contains("event: token"));
    assert!(body_str.contains("event: done"));

    unsafe {
        std::env::remove_var("CLERK_JWKS_URL");
        std::env::remove_var("CLERK_ISSUER");
    }
}
