# Multi-stage Dockerfile for Aevum & Bond blockchain

# Build stage
FROM rust:1.75-bookworm as builder

# Install system dependencies
RUN apt-get update && apt-get install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    protobuf-compiler \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN adduser --disabled-password --gecos '' --uid 1000 app

# Set working directory
WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./
COPY bond-core/Cargo.toml ./bond-core/
COPY aevum-core/Cargo.toml ./aevum-core/
COPY shared-crypto/Cargo.toml ./shared-crypto/
COPY p2p-network/Cargo.toml ./p2p-network/
COPY cli-tools/Cargo.toml ./cli-tools/
COPY wallet-desktop/Cargo.toml ./wallet-desktop/

# Create dummy source files for dependency caching
RUN mkdir -p bond-core/src aevum-core/src shared-crypto/src p2p-network/src cli-tools/src wallet-desktop/src
RUN echo "fn main() {}" > cli-tools/src/main.rs
RUN echo "fn main() {}" > wallet-desktop/src/main.rs
RUN echo "// dummy" > bond-core/src/lib.rs
RUN echo "// dummy" > aevum-core/src/lib.rs
RUN echo "// dummy" > shared-crypto/src/lib.rs
RUN echo "// dummy" > p2p-network/src/lib.rs

# Build dependencies (cached layer)
RUN cargo build --release
RUN rm -rf src/ target/release/deps/aevum_bond* target/release/deps/bond_core* target/release/deps/aevum_core* target/release/deps/shared_crypto* target/release/deps/p2p_network* target/release/deps/cli_tools* target/release/deps/wallet_desktop*

# Copy source code
COPY . .

# Build the application
RUN cargo build --release --bin cli-tools

# Create data directories
RUN mkdir -p /app/data/bond /app/data/aevum /app/config /app/logs
RUN chown -R app:app /app

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN adduser --disabled-password --gecos '' --uid 1000 app

# Create directories
RUN mkdir -p /app/data /app/config /app/logs
RUN chown -R app:app /app

# Copy binary from builder stage
COPY --from=builder /app/target/release/cli-tools /usr/local/bin/aevum-bond
COPY --from=builder --chown=app:app /app/data /app/data
COPY --from=builder --chown=app:app /app/config /app/config

# Create default configuration
RUN mkdir -p /app/config
COPY --chown=app:app <<EOF /app/config/default.toml
[network]
bond_port = 8080
aevum_port = 8081
p2p_port = 30303
bind_address = "0.0.0.0"

[security]
enable_tls = false
key_store_path = "/app/data/keystore"

[performance]
mining_threads = 4
cache_size = 1024

[logging]
level = "info"
output = "stdout"

[database]
bond_path = "/app/data/bond"
aevum_path = "/app/data/aevum"
EOF

# Switch to app user
USER app

# Set working directory
WORKDIR /app

# Expose ports
EXPOSE 8080 8081 30303

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=30s --retries=3 \
    CMD aevum-bond node status || exit 1

# Default command
CMD ["aevum-bond", "node", "start", "--config", "/app/config/default.toml"]

# Labels for metadata
LABEL org.opencontainers.image.title="Aevum & Bond Blockchain"
LABEL org.opencontainers.image.description="Post-quantum dual-ledger blockchain system"
LABEL org.opencontainers.image.source="https://github.com/ozzyjob/Aevum-Bond"
LABEL org.opencontainers.image.version="1.0.0"
LABEL org.opencontainers.image.licenses="MIT"
