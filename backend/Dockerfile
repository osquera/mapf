# Multi-stage build for Rust backend
FROM rust:1.76-slim AS builder

WORKDIR /app

# Install dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy workspace files
COPY Cargo.toml ./
COPY solvers ./solvers
COPY backend ./backend

# Build backend in release mode
WORKDIR /app/backend
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Copy binary from builder
COPY --from=builder /app/target/release/mapf-server /app/mapf-server

# Copy migrations
COPY backend/migrations /app/migrations

# Expose port
EXPOSE 3000

# Run the server
CMD ["/app/mapf-server"]
