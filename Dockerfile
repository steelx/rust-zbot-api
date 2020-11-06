# ------------------------------------------------------------------------------
# Cargo Build Stage
# ------------------------------------------------------------------------------

FROM rust:latest as builder
ENV DEBIAN_FRONTEND=noninteractive

WORKDIR     /rust
EXPOSE 8080

RUN apt-get update && apt-get install -y gcc-multilib pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

# create dummy application, s.t. cargo can download all dependencies
RUN         mkdir -p /rust/app/src && echo 'fn main(){}' > app/src/main.rs
WORKDIR     /rust/app

# Build & cache dependencies
COPY        Cargo.toml Cargo.lock ./
RUN         cargo build --release

# Copy application code
COPY        src ./src

# Build production binary
RUN         touch src/main.rs && cargo build --release

# Check if binary exists
RUN ls -l /rust/app/target/release/zbot_api

# Production container
FROM        scratch
COPY        --from=builder /rust/app/target/release/zbot_api /app
ENTRYPOINT  ["/app"]
