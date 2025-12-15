# dist_agent_lang Dockerfile
# Multi-stage build for optimized container image

# Stage 1: Build stage
FROM rust:1.70-slim as builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Install Node.js for build scripts
RUN curl -fsSL https://deb.nodesource.com/setup_18.x | bash - \
    && apt-get install -y nodejs

# Set working directory
WORKDIR /app

# Copy Cargo files
COPY Cargo.toml Cargo.lock ./
COPY package.json ./

# Copy source code
COPY src/ ./src/
COPY examples/ ./examples/
COPY scripts/ ./scripts/
COPY docs/ ./docs/

# Build the application
RUN cargo build --release

# Stage 2: Runtime stage
FROM debian:bullseye-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl1.1 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1000 dist_agent_lang

# Set working directory
WORKDIR /app

# Copy binary from builder stage
COPY --from=builder /app/target/release/dist_agent_lang /usr/local/bin/
RUN chmod +x /usr/local/bin/dist_agent_lang

# Copy examples and documentation
COPY --from=builder /app/examples/ ./examples/
COPY --from=builder /app/docs/ ./docs/

# Create configuration directory
RUN mkdir -p /app/config && chown -R dist_agent_lang:dist_agent_lang /app

# Switch to non-root user
USER dist_agent_lang

# Create default configuration
RUN mkdir -p /home/dist_agent_lang/.config/dist_agent_lang

# Environment variables
ENV DIST_AGENT_CONFIG_DIR=/app/config
ENV DIST_AGENT_LOG_LEVEL=info
ENV RUST_LOG=info

# Expose ports (if needed for web services)
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD dist_agent_lang --version || exit 1

# Default command
ENTRYPOINT ["dist_agent_lang"]

# Default arguments
CMD ["--help"]
