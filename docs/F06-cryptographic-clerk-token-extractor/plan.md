# Implementation Plan: F06. Cryptographic Clerk Token Extractor

This plan covers implementing the custom JWT extractor and JWKS in-memory cache on the Axum server.

## Prerequisites
*   `F05` Axum Server and CORS configuration must be complete.
*   Clerk environment configurations (`CLERK_JWKS_URL`) must be defined in the `.env` file.

## Phase 1: JWKS Fetching & Caching Engine
*   **Step 1: Implement JWKS Cache Structs** - Define key container structs mapping to JWKS payloads in `backend/src/claims.rs`. 
*   **Step 2: Implement Cached Key Retrieval** - Write helper methods that fetch Clerk's JWKS endpoints using `reqwest` and cache the public keys inside a `tokio::sync::RwLock` container, with a TTL check verifying age.

## Phase 2: Claims Extractor Integration
*   **Step 3: Implement Claims Extractor** - Add `impl<S> axum::extract::FromRequestParts<S> for Claims` returning custom error payloads when validation fails.
*   **Step 4: Write Unit/Integration Tests** - Create mock token validation tests inside `backend/src/claims.rs` to verify validation and parsing pipelines under correct, malformed, expired, and network-offline scenarios.
*   **Step 5: Export Claims Module** - Export `pub mod claims;` inside `backend/src/lib.rs` to expose the extractor for downstream services.
