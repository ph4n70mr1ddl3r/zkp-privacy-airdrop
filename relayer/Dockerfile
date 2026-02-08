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

# Build with optimizations
RUN touch src/main.rs && \
    cargo build --release --locked

# Runtime image
FROM debian:bookworm-slim

WORKDIR /app

RUN apt-get update && \
    apt-get install -y --no-install-recommends ca-certificates libssl3 && \
    rm -rf /var/lib/apt/lists/* && \
    useradd -r -u 1000 -s /sbin/nologin relayer

# Copy binary from builder
COPY --from=builder /app/target/release/zkp-airdrop-relayer /app/zkp-airdrop-relayer

# Change ownership
RUN chown -R relayer:relayer /app

USER relayer

EXPOSE 8080

HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD wget --no-verbose --tries=1 --spider http://localhost:8080/api/v1/health || exit 1

CMD ["/app/zkp-airdrop-relayer"]
