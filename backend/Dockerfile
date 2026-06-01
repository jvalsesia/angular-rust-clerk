# Stage 1: Build the Rust application
FROM rust:1-bookworm AS builder

WORKDIR /app

# Build dependencies only (leverage layer cache)
# Cache dependency compilation: copy manifests first, build a stub, then the
# real sources. Migrations are embedded into the binary via sqlx::migrate!,
# so they must be present at compile time.
COPY Cargo.toml Cargo.lock ./
RUN mkdir -p src \
    && echo "fn main() {}" > src/main.rs \
    && echo "" > src/lib.rs \
    && cargo build --release --locked 2>/dev/null || true \
    && rm -rf src

COPY src ./src

# Touch sources so the cached stub objects are rebuilt with real code.
RUN touch src/main.rs src/lib.rs \
    && cargo build --release --locked --bin backend

# ---- Runtime stage ----
FROM debian:bookworm-slim AS runtime

# rustls is used for TLS, but CA certificates are still needed to validate
# outbound HTTPS calls (LLM providers, embeddings, etc.).
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/*

# Run as a non-root user and give it a writable home for the file-backed
# secret store and any local state.
RUN useradd --create-home --uid 10001 appuser \
    && mkdir -p /data \
    && chown -R appuser:appuser /data

WORKDIR /app
COPY --from=builder /app/target/release/backend /usr/local/bin/backend

USER appuser

EXPOSE 3000

CMD ["backend"]
