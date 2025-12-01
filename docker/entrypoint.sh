#!/bin/bash
# ==============================================================================
# Tabrela - Container Entrypoint Script
# ==============================================================================
# This script runs at container startup to:
# 1. Process nginx config template with environment variables (PORT)
# 2. Start supervisord which manages all services
# ==============================================================================

set -e

# Default PORT to 8080 if not set by Railway
export PORT="${PORT:-8080}"

echo "Starting Tabrela backend services..."
echo "Nginx will listen on port: $PORT"

# Process nginx config template - substitute $PORT with actual value
# Only substitute PORT to avoid breaking other nginx variables like $host, $request_uri etc.
envsubst '${PORT}' < /etc/nginx/nginx.conf.template > /etc/nginx/nginx.conf

# Start supervisord (manages all services)
exec /usr/bin/supervisord -c /etc/supervisor/conf.d/supervisord.conf
