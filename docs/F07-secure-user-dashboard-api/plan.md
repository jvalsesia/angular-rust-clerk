# Implementation Plan: F07. Secure User Dashboard API

This plan covers implementing the protected Axum route handler, mounting it on the router, and writing integration tests.

## Prerequisites
- `F06` Cryptographic Clerk Token Extractor must be complete.

## Phase 1: Secure Endpoint Implementation
- **Step 1: Create Handler** - Implement the `get_user_profile` async handler in `backend/src/lib.rs` consuming the `Claims` extractor and returning user details.
- **Step 2: Add Route** - Add the `route("/api/user", get(get_user_profile))` mapping in the `create_app` Router setup.

## Phase 2: Integration Testing
- **Step 3: Build Secure Route Integration Tests** - Create `backend/tests/secure_routes_tests.rs` verifying that the GET `/api/user` endpoint successfully rejects requests with missing or malformed headers, and successfully returns user profiles when provided a valid signed JWT token.
