# Build stage
# Using nightly because some dependencies require edition2024 support
# NOTE: This Dockerfile is for LOCAL DEVELOPMENT/TESTING only.
# Production deployment uses the root Dockerfile which builds all services.
#
# Build from repo root: docker build -f services/auth/Dockerfile -t auth-service .
FROM rustlang/rust:nightly-slim AS builder

# Install required dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /app

# Copy manifests (from repo root context)
COPY services/auth/Cargo.toml ./
COPY services/auth/Cargo.loc[k] ./

# Copy source code
COPY services/auth/src ./src
# Migrations are shared across services
COPY services/migrations ./migrations

# Build for release
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -m -u 1001 appuser

WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/auth /app/auth
COPY --from=builder /app/migrations /app/migrations

# Change ownership
RUN chown -R appuser:appuser /app

# Switch to app user
USER appuser

# Expose port
EXPOSE 8081

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8081/health || exit 1

# Run the binary
CMD ["./auth"]
