#!/bin/bash
# ==============================================================================
# Tabrela - Container Entrypoint Script
# ==============================================================================
# This script runs at container startup to:
# 1. Run database migrations
# 2. Process nginx config template with environment variables
# 3. Start supervisord which manages all services
# ==============================================================================

set -e

# Default PORT to 8080 if not set by Railway
export PORT="${PORT:-8080}"

# Default ALLOWED_ORIGIN for CORS - can be overridden via environment variable
# For production, set this to your frontend domain (e.g., https://tabrela.giki-dt.com)
export ALLOWED_ORIGIN="${ALLOWED_ORIGIN:-*}"

# Git commit SHA for version tracking
# Railway provides RAILWAY_GIT_COMMIT_SHA automatically
# Fall back to GIT_COMMIT_SHA (from Docker build arg) or "unknown"
export GIT_COMMIT_SHA="${RAILWAY_GIT_COMMIT_SHA:-${GIT_COMMIT_SHA:-unknown}}"

echo "Starting Tabrela backend services..."
echo "Nginx will listen on port: $PORT"
echo "CORS allowed origin: $ALLOWED_ORIGIN"
echo "Git commit SHA: $GIT_COMMIT_SHA"

# Run database migrations before starting services
# This ensures the database schema is up to date before any service tries to connect
echo "Running database migrations..."
cd /app/migrations
if /app/bin/sqlx migrate run --database-url "$DATABASE_URL"; then
    echo "Migrations completed successfully"
else
    echo "Warning: Migration failed, services will attempt migration on startup"
fi
cd /app

# Process nginx config template - substitute environment variables
# Only substitute specific variables to avoid breaking nginx variables like $host, $request_uri
envsubst '${PORT} ${ALLOWED_ORIGIN} ${GIT_COMMIT_SHA}' < /etc/nginx/nginx.conf.template > /etc/nginx/nginx.conf

# Start supervisord (manages all services)
exec /usr/bin/supervisord -c /etc/supervisor/conf.d/supervisord.conf
