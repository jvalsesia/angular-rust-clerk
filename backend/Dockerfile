# Stage 1: Build the Rust application
FROM rust:1.96-slim AS builder
WORKDIR /usr/src/app

# Install system build dependencies
RUN apt-get update && apt-get install -y pkg-config libssl-dev ca-certificates && rm -rf /var/lib/apt/lists/*

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Build dependencies only (leverage layer cache)
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -f src/main.rs

# Copy actual source code
COPY src ./src

# Touch main.rs to trigger rebuild of code
RUN touch src/main.rs && cargo build --release

# Stage 2: Create a minimal runtime container
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/local/bin
COPY --from=builder /usr/src/app/target/release/backend /usr/local/bin/backend

EXPOSE 3000
CMD ["/usr/local/bin/backend"]
