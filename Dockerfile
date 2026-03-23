# Multi-stage build for the DORA EU server (Axum + SQLx)

# ---- Builder stage ----
# Use nightly to support crates that currently require edition2024 in Cargo
FROM rustlang/rust:nightly AS builder

WORKDIR /app

# Install system dependencies needed by sqlx native-tls and builds
RUN apt-get update \
    && apt-get install -y --no-install-recommends pkg-config libssl-dev ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Pre-cache dependencies
COPY Cargo.toml ./
COPY src ./src
COPY web ./web
COPY schema ./schema

# Install rustowl CLI (so it's available in the runtime PATH)
RUN cargo install rustowl

RUN cargo build --release

# ---- Runtime stage ----
FROM debian:bookworm-slim AS runtime

WORKDIR /app

# Install runtime deps for native-tls and HTTPS
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates libssl3 \
    && rm -rf /var/lib/apt/lists/* \
    && update-ca-certificates

# Copy binary and static web assets
COPY --from=builder /app/target/release/DORAEU /app/DORAEU
COPY --from=builder /app/web /app/web
# Provide rustowl in PATH inside the runtime image
COPY --from=builder /usr/local/cargo/bin/rustowl /usr/local/bin/rustowl

EXPOSE 8080

# Expect DATABASE_URL to be provided at runtime
ENV RUST_LOG=info

CMD ["/app/DORAEU", "serve", "--bind", "0.0.0.0:8080"]
