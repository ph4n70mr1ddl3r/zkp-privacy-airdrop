FROM rust:1.75-slim as builder

WORKDIR /app

# Copy Cargo files
COPY Cargo.toml Cargo.lock ./

# Create dummy main.rs to cache dependencies
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# Copy actual source
COPY src ./src

# Build
RUN touch src/main.rs && \
    cargo build --release

# Runtime image
FROM debian:bookworm-slim

WORKDIR /app

RUN apt-get update && \
    apt-get install -y ca-certificates libssl3 && \
    rm -rf /var/lib/apt/lists/*

# Copy binary from builder
COPY --from=builder /app/target/release/zkp-airdrop-relayer /app/zkp-airdrop-relayer

EXPOSE 8080

CMD ["/app/zkp-airdrop-relayer"]
