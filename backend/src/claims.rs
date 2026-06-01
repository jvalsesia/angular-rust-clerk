use std::sync::OnceLock;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

use axum::{
    extract::FromRequestParts,
    http::{header::AUTHORIZATION, request::Parts, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};

/// Custom JSON Web Key structure matching Clerk's response format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwkKey {
    pub kid: String,
    pub kty: String,
    pub alg: String,
    pub n: String,
    pub e: String,
}

/// A JWKS response payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Jwks {
    pub keys: Vec<JwkKey>,
}

struct JwksCache {
    keys: Vec<JwkKey>,
    fetched_at: Option<Instant>,
}

impl JwksCache {
    fn new() -> Self {
        Self {
            keys: Vec::new(),
            fetched_at: None,
        }
    }

    fn is_expired(&self) -> bool {
        match self.fetched_at {
            None => true,
            Some(fetched_at) => fetched_at.elapsed() >= Duration::from_secs(24 * 3600),
        }
    }
}

// Thread-safe in-memory cache for public keys
static JWKS_CACHE: OnceLock<RwLock<JwksCache>> = OnceLock::new();

/// Decoded JWT claim data.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub exp: u64,
    pub iss: String,
    // Add other clerk-specific fields if needed in the future
}

/// Potential rejection types when parsing JWT claims from headers.
#[derive(Debug)]
pub enum ClaimsRejection {
    MissingHeader,
    InvalidFormat,
    Expired,
    InvalidSignature,
    ProviderOffline,
}

impl IntoResponse for ClaimsRejection {
    fn into_response(self) -> Response {
        let (status, error_msg) = match self {
            ClaimsRejection::MissingHeader => (
                StatusCode::UNAUTHORIZED,
                "Missing authorization header",
            ),
            ClaimsRejection::InvalidFormat => (
                StatusCode::BAD_REQUEST,
                "Invalid authorization format",
            ),
            ClaimsRejection::Expired => (
                StatusCode::UNAUTHORIZED,
                "Token expired",
            ),
            ClaimsRejection::InvalidSignature => (
                StatusCode::UNAUTHORIZED,
                "Invalid token or signature",
            ),
            ClaimsRejection::ProviderOffline => (
                StatusCode::SERVICE_UNAVAILABLE,
                "Auth provider offline",
            ),
        };

        let body = serde_json::json!({ "error": error_msg });
        (status, Json(body)).into_response()
    }
}

/// Fetches Clerk JWKS, updating the cache if expired or missing.
async fn get_jwk_key(kid: &str, jwks_url: &str) -> Result<JwkKey, ClaimsRejection> {
    let cache_lock = JWKS_CACHE.get_or_init(|| RwLock::new(JwksCache::new()));

    // Try reading first
    {
        let cache = cache_lock.read().await;
        if !cache.is_expired() {
            let key_opt = cache.keys.iter().find(|k| k.kid == kid);
            if let Some(key) = key_opt {
                return Ok(key.clone());
            }
        }
    }

    // Cache is expired or key not found: acquire write lock to refresh
    let mut cache = cache_lock.write().await;
    
    // Double check state inside write lock
    if cache.is_expired() || cache.keys.iter().all(|k| k.kid != kid) {
        let response = reqwest::get(jwks_url)
            .await
            .map_err(|_| ClaimsRejection::ProviderOffline)?;

        let jwks = response
            .json::<Jwks>()
            .await
            .map_err(|_| ClaimsRejection::ProviderOffline)?;

        cache.keys = jwks.keys;
        cache.fetched_at = Some(Instant::now());
    }

    cache
        .keys
        .iter()
        .find(|k| k.kid == kid)
        .cloned()
        .ok_or(ClaimsRejection::InvalidSignature)
}

/// Internal validation logic.
fn validate_token(
    token: &str,
    jwk: &JwkKey,
    expected_issuer: &str,
) -> Result<Claims, ClaimsRejection> {
    let decoding_key = DecodingKey::from_rsa_components(&jwk.n, &jwk.e)
        .map_err(|_| ClaimsRejection::InvalidSignature)?;

    let mut validation = Validation::new(Algorithm::RS256);
    validation.set_issuer(&[expected_issuer]);

    let token_data = decode::<Claims>(token, &decoding_key, &validation).map_err(|err| {
        match err.kind() {
            jsonwebtoken::errors::ErrorKind::ExpiredSignature => ClaimsRejection::Expired,
            _ => ClaimsRejection::InvalidSignature,
        }
    })?;

    Ok(token_data.claims)
}

impl<S> FromRequestParts<S> for Claims
where
    S: Send + Sync,
{
    type Rejection = ClaimsRejection;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // Extract Authorization header
        let auth_header = parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|val| val.to_str().ok())
            .ok_or(ClaimsRejection::MissingHeader)?;

        if !auth_header.starts_with("Bearer ") {
            return Err(ClaimsRejection::InvalidFormat);
        }

        let token = &auth_header["Bearer ".len()..];

        // Retrieve JWT header to extract key id (kid)
        let header = decode_header(token).map_err(|_| ClaimsRejection::InvalidSignature)?;
        let kid = header.kid.ok_or(ClaimsRejection::InvalidSignature)?;

        // Read environment variables at request time to support configuration dynamic updates
        let jwks_url = std::env::var("CLERK_JWKS_URL")
            .unwrap_or_else(|_| "https://api.clerk.com/v1/jwks".to_string());
        let issuer = std::env::var("CLERK_ISSUER")
            .unwrap_or_else(|_| "https://gentle-ophaph-98.clerk.accounts.dev".to_string());

        let jwk = get_jwk_key(&kid, &jwks_url).await?;
        validate_token(token, &jwk, &issuer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // A valid 2048-bit RSA Private Key PEM for test generation
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

    #[test]
    fn test_token_validation_failure() {
        let invalid_jwk = JwkKey {
            kid: "test_kid".to_string(),
            kty: "RSA".to_string(),
            alg: "RS256".to_string(),
            n: "invalid_modulus".to_string(),
            e: "AQAB".to_string(),
        };

        // If we validate an empty or malformed token, it must fail
        let result = validate_token("invalid_token", &invalid_jwk, "https://issuer.com");
        assert!(matches!(result, Err(ClaimsRejection::InvalidSignature)));
    }

    #[test]
    fn test_token_validation_success() {
        use jsonwebtoken::{encode, Header, EncodingKey};
        use std::time::SystemTime;

        let my_claims = Claims {
            sub: "user_456".to_string(),
            exp: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs()
                + 3600,
            iss: "https://issuer.com".to_string(),
        };

        let mut header = Header::new(Algorithm::RS256);
        header.kid = Some("test_kid".to_string());

        let encoding_key = EncodingKey::from_rsa_pem(TEST_PRIVATE_KEY_PEM).unwrap();
        let token = encode(&header, &my_claims, &encoding_key).unwrap();

        let valid_jwk = JwkKey {
            kid: "test_kid".to_string(),
            kty: "RSA".to_string(),
            alg: "RS256".to_string(),
            n: "pzXLHzyHBItcRNc_TaK03W2mk0msNugESzDqXzgv_F2HpbcYc7-m6xsz7n7ufHnd17ragHxCMjKjy_fQYps05kqIqwFt2GECcgQ1uRvnX5Ob9NUAqITmK-qYXRd7ubasS0xpVV4a5TJGfpXfHm8za3qWXI5-qMqXbjwUeIe8s9McRFhoGEeUUuFyTVuUebsEVyUps7GG-a6Yk2hRUPESPD7d-Mzw5efAU0svVe4bYxcw51kGQynlae8NsS5T56bTYC4oUxEfqRIeozy2ZXXIegBvObXTw7X-D2U_OYhh15pYFGq1SvDoftYktdpwTqcps1ac1os58d8kh8DfVi6JFw".to_string(),
            e: "AQAB".to_string(),
        };

        let result = validate_token(&token, &valid_jwk, "https://issuer.com");
        assert!(result.is_ok());
        let claims = result.unwrap();
        assert_eq!(claims.sub, "user_456");
    }
}
