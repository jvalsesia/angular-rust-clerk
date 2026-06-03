# Project Intent: Modern Full-Stack Template (Angular + Axum + Clerk + LiteLLM Proxy)

## 1. Project Overview

The objective of this project is to build a high-performance, ultra-secure, full-stack boilerplate application. This project serves as an architectural blueprint for combining modern frontend reactivity with blazingly fast, memory-safe backend execution.

### Key Objectives

* **Zero-Overhead Reactivity:** Leverage Angular’s latest native Signals and zoneless architecture.
* **Compile-Time Type Safety:** Utilize Axum’s macro-free, type-safe extractor routing mechanism.
* **Turnkey Authentication:** Offload identity management, sign-ins, and sign-ups completely to Clerk.
* **AI Integration Ready:** Provide a high-performance LLM chat interface backed by a unified routing proxy.

---

## 2. Architectural Design & Tech Stack

### Frontend (Angular 21+)

* **Reactivity:** Fully **Zoneless change detection** powered exclusively by **Angular Signals**.
* **Build Pipeline:** Vite-powered modern `application` builder (replacing traditional Webpack setups).
* **Testing:** **Vitest** for blistering fast, modern unit testing.
* **Auth UI:** Integrated via the official Clerk JavaScript / Angular SPA integration to handle Sign-In and Sign-Up flows seamlessly.
* **AI Chat Component:** A streaming, signal-driven chat interface built with native modern forms.

### Backend (Rust Axum 0.8+)

* **Runtime:** `tokio` (Async Rust 2024 Edition).
* **Routing & Logic:** `axum` utilizing type-safe extractors (`Json<T>`, `State<S>`).
* **Middleware:** `tower-http` for production-grade CORS configuration and logging (`tracing`).
* **Auth Validation:** Cryptographic verification of Clerk-issued JSON Web Tokens (JWTs) using `jsonwebtoken` or `reqwest` for JWKS fetching.
* **AI Router:** Forwarding chat prompts securely to LiteLLM Proxy using an async HTTP client (e.g., `reqwest`) supporting SSE (Server-Sent Events) for real-time response streaming.

### Infrastructure & Proxy Layer

* **AI Gateway:** **LiteLLM Proxy** acting as a unified OpenAI-compatible gateway to abstract, load-balance, and manage API keys for underlying LLM providers (e.g., Anthropic, OpenAI, or local Ollama instances).

---

## 3. Core System User Flow

```mermaid
sequenceDiagram
    autonumber
    actor Client as Angular Client
    participant Clerk as Clerk Auth API
    participant Backend as Axum Backend (Rust)
    participant LiteLLM as LiteLLM Proxy
    participant LLM as LLM Provider

    Client->>Clerk: Render Sign-In/Up
    Clerk-->>Client: Auth Success (Get JWT)
    
    rect rgb(240, 240, 240)
        Note over Client, Backend: Secure LLM Chat Interaction
        Client->>Backend: Post Prompt (with JWT)
        Backend->>Clerk: Validate JWT via JWKS
        Backend->>LiteLLM: Forward Chat Request (OpenAI Spec)
        LiteLLM->>LLM: Route to Target Provider
        LLM-->>LiteLLM: Stream Tokens (SSE)
        LiteLLM-->>Backend: Stream Response
        Backend-->>Client: Stream Type-Safe Final Response
    end

```

---

## 4. Implementation Phase & Milestones

### Phase 1: Clerk Set Up & Initial Frontend

* Initialize an Angular 21 project without `zone.js` (`provideExperimentalZonelessChangeDetection()`).
* Create a clean layout containing a Landing Page, a Login/Signup Screen, and a Protected Dashboard area.
* Inject Clerk’s SPA SDK. Embed `<clerk-sign-in>` and `<clerk-sign-up>` components onto the login screen.
* Implement an Angular functional Route Guard that queries a global Signal-based `AuthService` to allow/deny access to the dashboard.

### Phase 2: High-Performance Backend Setup

* Initialize a Cargo workspace targeting the **Rust 2024 Edition**.
* Set up an Axum 0.8 base server with new path syntax matching `/{id}` directly, omitting raw string macros.
* Implement a custom Axum extractor (e.g., `struct Claims`) implementing `FromRequestParts`. This extractor will automatically extract the `Authorization: Bearer <token>` header, decode it using Clerk’s public JSON Web Key Sets (JWKS), and reject unauthorized requests before they ever touch your route handlers.

### Phase 3: LiteLLM Integration & Chat Endpoint

* Setup a containerized **LiteLLM Proxy** instance configuring backend models and securely managing upstream API keys.
* Create a protected `/api/chat` route in Axum requiring valid Clerk `Claims`.
* Implement streaming token proxying in Axum using `axum::response::sse::Sse` to forward text streams from LiteLLM directly back to the client without blocking.
* Build an Angular reactive chat window interface using modern Signals to seamlessly append incoming text fragments to the UI state.

### Phase 4: Integration & End-to-End Testing

* Configure the Angular `HttpInterceptor` to automatically attach the Clerk JWT token to all outbound backend calls.
* Configure `tower-http::cors::CorsLayer` in Rust to allow local communication during development.
* Verify that registering or logging in through Clerk correctly cascades access to the Axum backend endpoints and authorizes the user to prompt the LLM.

---

## 5. Risks & Strategic Solutions

* **Clerk Backend Ecosystem:** Clerk does not distribute a first-party, officially maintained *Rust* SDK.
* *Solution:* We will write a lightweight custom Axum middleware layer using the `jsonwebtoken` crate. This approach is highly performant because it relies on local cryptographic decoding using Clerk's hosted JWKS keys, avoiding a blocking network call on every API request.
* **Evolving Framework Semantics:** Axum 0.8 introduces cleaner trait handling and compilation errors, but relies heavily on correct layer ordering.
* *Solution:* Ensure the `CorsLayer` wraps outside of the authorization extractors to prevent pre-flight `OPTIONS` requests from dropping due to a missing auth token.
* **Streaming Overhead and Connection Drops:** Managing stateful Server-Sent Events (SSE) from LiteLLM through Axum down to Angular can leave dangling connections if a client closes the tab mid-generation.
* *Solution:* Use `tokio::select!` in the Axum handler to monitor the client's connection dropped state (`axum::extract::Request` extensions or stream cancellation tracking) to immediately abort the upstream connection to LiteLLM, saving API tokens.

---
