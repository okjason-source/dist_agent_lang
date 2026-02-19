# dist_agent_lang — production Docker image for AI + blockchain
#
#   • Run scripts:    docker run --rm -v $(pwd):/app/work your-image run /app/work/script.dal
#   • HTTP server:   docker run -p 8080:8080 -e OPENAI_API_KEY=... your-image serve app.dal --port 8080
#   • Blockchain:    set DAL_RPC_URL, DAL_PRIVATE_KEY, DAL_MOLD_REGISTRY_ADDRESS (via env or secrets)
#   • Persistence:   mount /app/config, /app/logs, and optionally /app/data for tx storage
#
# Build with default features (HTTP + Web3 + SQLite):  docker build -t dal .
# Build minimal (CLI only):  docker build --build-arg FEATURES= -t dal .

# -----------------------------------------------------------------------------
# Stage 1: Builder
# -----------------------------------------------------------------------------
FROM rust:1-bookworm-slim AS builder

# Build-time: enable HTTP interface, Web3/EVM, SQLite (for AI agents + chain + tx persistence)
ARG FEATURES=http-interface,web3,sqlite-storage
ARG CARGO_BUILD_JOBS=

RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src/ ./src/
COPY examples/ ./examples/

RUN cargo build --release --no-default-features --features "$FEATURES"

# -----------------------------------------------------------------------------
# Stage 2: Runtime
# -----------------------------------------------------------------------------
FROM debian:bookworm-slim

# Runtime deps only
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# OCI labels (helpful for registries and ops)
LABEL org.opencontainers.image.source="https://github.com/okjason-source/dist_agent_lang"
LABEL org.opencontainers.image.title="dist_agent_lang (dal)"
LABEL org.opencontainers.image.description="CLI and runtime for dist_agent_lang — AI agents and blockchain"

RUN useradd -m -u 1000 -s /bin/sh dal

WORKDIR /app

COPY --from=builder /app/target/release/dal /usr/local/bin/
RUN chmod +x /usr/local/bin/dal

COPY --from=builder /app/examples/ ./examples/

# Writable dirs for config, logs, and optional tx/key storage (mount in production)
RUN mkdir -p /app/config /app/logs /app/data && chown -R dal:dal /app

USER dal

# --- Environment (set at run time; no secrets here) ---
# AI: OPENAI_API_KEY, ANTHROPIC_API_KEY, OPENAI_MODEL, DAL_AI_ENDPOINT, DAL_AI_MODEL, DAL_AI_MAX_TOKENS
# Chain: DAL_RPC_URL, DAL_PRIVATE_KEY, DAL_MOLD_REGISTRY_ADDRESS, CHAIN_ASSET_CHAIN_ID
# Tx:   DAL_TX_STORAGE=sqlite, DAL_TX_STORAGE_PATH=/app/data/tx.db
# Keys: DAL_KEY_STORE=file, DAL_KEY_STORE_PATH=/app/config/keys
# Log:  LOG_SINK=file, LOG_DIR=/app/logs, LOG_LEVEL=info
# Auth: JWT_SECRET (for serve), AUTH_RATE_LIMIT_*
ENV DIST_AGENT_CONFIG_DIR=/app/config
ENV DIST_AGENT_LOG_LEVEL=info
ENV RUST_LOG=info
ENV LOG_DIR=/app/logs
ENV LOG_SINK=file

EXPOSE 8080

VOLUME ["/app/config", "/app/logs", "/app/data"]

HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD dal --version || exit 1

ENTRYPOINT ["dal"]
CMD ["--help"]
