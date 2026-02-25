# Build stage
FROM rustlang/rust:nightly-bookworm AS builder

# Install required dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /app

# Copy manifests first to cache dependencies
COPY Cargo.toml Cargo.lock* ./

# Create dummy source to build dependencies
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    echo "pub fn lib() {}" > src/lib.rs

# Build dependencies (this layer will be cached)
RUN cargo build --release && \
    rm -rf src target/release/backend-rust target/release/backend-rust.d

# Copy actual source code
COPY . .

# Build the application (only app code, deps are cached)
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    libpq5 \
    libssl3 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /app

# Copy the binary from builder
COPY --from=builder /app/target/release/backend-rust /app/backend-rust

# Copy migrations
COPY --from=builder /app/migrations /app/migrations
COPY --from=builder /app/diesel.toml /app/diesel.toml

# Expose port
EXPOSE 8080

# Run the binary
CMD ["/app/backend-rust"]
