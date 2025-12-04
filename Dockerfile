# ==============================================================================
# Tabrela Backend - Multi-Service Docker Image
# ==============================================================================
# This Dockerfile builds all backend services into a single image:
# - auth (Rust) - port 8081 - implements GET /health endpoint
# - attendance (Rust) - port 8082 - implements GET /health endpoint
# - merit (Rust) - port 8083 - implements GET /health endpoint
# - tabulation (Rust) - port 8084 - implements GET /health endpoint
# - email (Python) - port 5000 - internal only, implements GET /health endpoint
# - webhook (Python) - port 5001 - handles Railway deploy webhooks
#
# Uses multi-stage builds to minimize final image size
#
# Build args:
# - GIT_COMMIT_SHA: Git commit SHA for version tracking (passed by CI/CD)
#
# IMPORTANT: All Rust services must implement a GET /health endpoint that
# returns 200 OK for health checks to pass. See services/*/src/main.rs
# ==============================================================================

# Python version - change this if updating the Python base image
ARG PYTHON_VERSION=3.11
# Rust version - use "latest" for most recent stable, or pin to specific version
ARG RUST_VERSION=1.90
# Git commit SHA for version tracking - passed from CI/CD
ARG GIT_COMMIT_SHA=unknown

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
COPY services/tabulation/Cargo.toml ./tabulation/

# Copy source code
COPY services/auth/src ./auth/src
COPY services/attendance/src ./attendance/src
COPY services/merit/src ./merit/src
COPY services/tabulation/src ./tabulation/src

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

# Build tabulation service
WORKDIR /build/tabulation
RUN cargo build --release

# Install sqlx-cli for running migrations at container startup
# Use CARGO_HOME to ensure we know where binaries are installed
ENV CARGO_HOME=/usr/local/cargo
RUN cargo install sqlx-cli --no-default-features --features native-tls,postgres

# ==============================================================================
# Stage 2: Build Python email service
# ==============================================================================
ARG PYTHON_VERSION
FROM python:${PYTHON_VERSION}-slim AS python-builder

WORKDIR /build/email

# Create virtual environment at the SAME PATH it will be used in runtime
# This is critical because venv hardcodes the Python path in script shebangs
RUN python -m venv /app/email/venv
ENV PATH="/app/email/venv/bin:$PATH"

COPY services/email/requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

COPY services/email/app.py .
COPY services/email/models.py .

# Build webhook service
WORKDIR /build/webhook
RUN python -m venv /app/webhook/venv
ENV PATH="/app/webhook/venv/bin:$PATH"

COPY services/webhook/requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

COPY services/webhook/app.py .

# ==============================================================================
# Stage 3: Final runtime image
# ==============================================================================
# Using Python slim image since we need Python for email service
# This ensures the venv works correctly (same Python paths)
ARG PYTHON_VERSION
FROM python:${PYTHON_VERSION}-slim AS runtime

# Re-declare ARG after FROM to use in this stage
ARG GIT_COMMIT_SHA=unknown
# Set Git commit SHA as environment variable for version tracking
ENV GIT_COMMIT_SHA=${GIT_COMMIT_SHA}

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
COPY --from=rust-builder /build/tabulation/target/release/tabulation /app/bin/tabulation
COPY --from=rust-builder /usr/local/cargo/bin/sqlx /app/bin/sqlx

# Copy Python virtual environment (must be at same path as created in builder)
COPY --from=python-builder /app/email/venv /app/email/venv

# Copy Python app files
COPY --from=python-builder /build/email/app.py /app/email/app.py
COPY --from=python-builder /build/email/models.py /app/email/models.py

# Copy webhook service
COPY --from=python-builder /app/webhook/venv /app/webhook/venv
COPY --from=python-builder /build/webhook/app.py /app/webhook/app.py

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
