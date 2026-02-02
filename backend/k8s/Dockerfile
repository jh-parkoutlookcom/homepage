# Multi-stage build for Rust Application
FROM rust:1-slim-trixie AS builder 
WORKDIR /app

# Copy workspace files
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates

# Build the application
RUN cargo build --release -p web-server


# Runtime stage
FROM debian:trixie-slim
# Install CA certificates and required libraries
RUN apt-get update && \
    apt-get install -y ca-certificates libssl3 && \
    rm -rf /var/lib/apt/lists/*
WORKDIR /app 
COPY --from=builder /app/target/release/web-server /app/web-server
EXPOSE 8080 50051
CMD ["/app/web-server"]