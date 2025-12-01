# ==============================================================================
# Tabrela Backend - Multi-Service Docker Image
# ==============================================================================
# This Dockerfile builds all backend services into a single image:
# - auth (Rust) - port 8081 - implements GET /health endpoint
# - attendance (Rust) - port 8082 - implements GET /health endpoint
# - merit (Rust) - port 8083 - implements GET /health endpoint
# - email (Python) - port 5000 - internal only, implements GET /health endpoint
#
# Uses multi-stage builds to minimize final image size
#
# IMPORTANT: All Rust services must implement a GET /health endpoint that
# returns 200 OK for health checks to pass. See services/*/src/main.rs
# ==============================================================================

# Python version - change this if updating the Python base image
ARG PYTHON_VERSION=3.11
# Rust version - use "latest" for most recent stable, or pin to specific version
ARG RUST_VERSION=1.90

# ==============================================================================
# Stage 1: Build Rust services
# ==============================================================================
FROM rust:${RUST_VERSION}-bookworm AS rust-builder

# Install dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /build

# Copy workspace Cargo files
COPY services/auth/Cargo.toml services/auth/Cargo.lock ./auth/
COPY services/attendance/Cargo.toml services/attendance/Cargo.lock ./attendance/
COPY services/merit/Cargo.toml services/merit/Cargo.lock ./merit/

# Copy source code
COPY services/auth/src ./auth/src
COPY services/attendance/src ./attendance/src
COPY services/merit/src ./merit/src

# Copy migrations (required by sqlx::migrate! macro at compile time)
# Path must be ../migrations relative to /build/auth, so we put it at /build/migrations
COPY services/migrations ./migrations

# Build auth service
WORKDIR /build/auth
RUN cargo build --release

# Build attendance service
WORKDIR /build/attendance
RUN cargo build --release

# Build merit service
WORKDIR /build/merit
RUN cargo build --release

# ==============================================================================
# Stage 2: Build Python email service
# ==============================================================================
ARG PYTHON_VERSION
FROM python:${PYTHON_VERSION}-slim AS python-builder

WORKDIR /build/email

# Create virtual environment for clean dependency isolation
RUN python -m venv /opt/venv
ENV PATH="/opt/venv/bin:$PATH"

COPY services/email/requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

COPY services/email/app.py .
COPY services/email/models.py .

# ==============================================================================
# Stage 3: Final runtime image
# ==============================================================================
# Using Python slim image since we need Python for email service
# This ensures the venv works correctly (same Python paths)
ARG PYTHON_VERSION
FROM python:${PYTHON_VERSION}-slim AS runtime

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    supervisor \
    curl \
    nginx \
    gettext-base \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -m -u 1001 appuser

WORKDIR /app

# Copy Rust binaries from builder
COPY --from=rust-builder /build/auth/target/release/auth /app/bin/auth
COPY --from=rust-builder /build/attendance/target/release/attendance /app/bin/attendance
COPY --from=rust-builder /build/merit/target/release/merit /app/bin/merit

# Copy Python virtual environment (version-independent path)
COPY --from=python-builder /opt/venv /app/email/venv

# Copy Python app files
COPY --from=python-builder /build/email/app.py /app/email/app.py
COPY --from=python-builder /build/email/models.py /app/email/models.py

# Copy migrations
COPY services/migrations /app/migrations

# Copy supervisor config
COPY docker/supervisord.conf /etc/supervisor/conf.d/supervisord.conf

# Copy nginx config template (will be processed by envsubst at startup)
COPY docker/nginx.conf /etc/nginx/nginx.conf.template

# Copy entrypoint script
COPY docker/entrypoint.sh /app/entrypoint.sh
RUN chmod +x /app/entrypoint.sh

# Copy health check script
COPY docker/healthcheck.sh /app/healthcheck.sh
RUN chmod +x /app/healthcheck.sh

# Create log directory
RUN mkdir -p /var/log/tabrela && chown -R appuser:appuser /var/log/tabrela

# Make binaries executable
RUN chmod +x /app/bin/*

# Change ownership
RUN chown -R appuser:appuser /app

# Expose ports
# Only nginx is exposed - it reverse proxies to internal services
# Railway will set PORT env var (usually 8080 or similar)
# Internal: Auth: 8081, Attendance: 8082, Merit: 8083, Email: 5000
EXPOSE 8080

# Health check - checks ALL services, not just auth
# Returns unhealthy if any service is down
# Increased start-period to allow all services time to initialize
HEALTHCHECK --interval=30s --timeout=10s --start-period=30s --retries=3 \
    CMD /app/healthcheck.sh || exit 1

# Run entrypoint script which:
# 1. Processes nginx config with PORT env var
# 2. Starts supervisord to manage all services
#
# Note: supervisord runs as root to manage child processes, but individual
# services run as 'appuser' for security (see supervisord.conf)
CMD ["/app/entrypoint.sh"]
