#!/bin/bash
# ==============================================================================
# Tabrela - Container Entrypoint Script
# ==============================================================================
# This script runs at container startup to:
# 1. Process nginx config template with environment variables (PORT, ALLOWED_ORIGIN)
# 2. Start supervisord which manages all services
# ==============================================================================

set -e

# Default PORT to 8080 if not set by Railway
export PORT="${PORT:-8080}"

# Default ALLOWED_ORIGIN for CORS - can be overridden via environment variable
# Format: single origin or comma-separated list, but nginx uses single value
# For production, set this to your frontend domain (e.g., https://tabrela.giki-dt.com)
export ALLOWED_ORIGIN="${ALLOWED_ORIGIN:-*}"

echo "Starting Tabrela backend services..."
echo "Nginx will listen on port: $PORT"
echo "CORS allowed origin: $ALLOWED_ORIGIN"

# Process nginx config template - substitute environment variables
# Only substitute specific variables to avoid breaking nginx variables like $host, $request_uri
envsubst '${PORT} ${ALLOWED_ORIGIN}' < /etc/nginx/nginx.conf.template > /etc/nginx/nginx.conf

# Start supervisord (manages all services)
exec /usr/bin/supervisord -c /etc/supervisor/conf.d/supervisord.conf
