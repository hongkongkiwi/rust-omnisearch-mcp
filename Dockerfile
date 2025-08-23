# Multi-stage build for optimal image size
FROM rust:1.75-slim as builder

# Install system dependencies required for building
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -m -u 1001 appuser

# Set work directory
WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./
COPY benches ./benches
COPY tests ./tests

# Create a dummy main.rs to cache dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Build dependencies (this step is cached unless Cargo.toml changes)
RUN cargo build --release --bin omnisearch-mcp
RUN rm src/main.rs

# Copy source code
COPY src ./src
COPY config.toml ./

# Build application
RUN cargo build --release --bin omnisearch-mcp

# Remove debug symbols and strip binary
RUN strip target/release/omnisearch-mcp

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -m -u 1001 appuser

# Set work directory
WORKDIR /app

# Copy configuration files
COPY config.toml ./
COPY --from=builder /app/target/release/omnisearch-mcp ./

# Create config directory and set permissions
RUN mkdir -p /app/config && \
    chown -R appuser:appuser /app

# Switch to non-root user
USER appuser

# Expose port (default from config.toml)
EXPOSE 3000

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3000/health || exit 1

# Run the application
CMD ["./omnisearch-mcp"]

# Add labels
LABEL org.opencontainers.image.title="Omnisearch MCP"
LABEL org.opencontainers.image.description="A comprehensive MCP server providing unified search capabilities"
LABEL org.opencontainers.image.vendor="hongkongkiwi"
LABEL org.opencontainers.image.licenses="MIT"
LABEL org.opencontainers.image.source="https://github.com/hongkongkiwi/rust-omnisearch-mcp"