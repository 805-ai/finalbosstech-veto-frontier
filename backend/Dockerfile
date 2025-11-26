# Multi-stage Dockerfile for Rust backend
# Stage 1: Build
FROM rust:1.75-slim as builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Create dummy main to cache dependencies
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# Copy source code
COPY src ./src

# Build for release (remove dummy artifacts first)
RUN rm ./target/release/deps/veto_frontier_backend* && \
    cargo build --release

# Stage 2: Runtime
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libpq5 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/veto-frontier-backend /usr/local/bin/veto-frontier-backend

# Create non-root user
RUN useradd -m -u 1000 veto && \
    chown -R veto:veto /app

USER veto

EXPOSE 8888

CMD ["veto-frontier-backend"]
