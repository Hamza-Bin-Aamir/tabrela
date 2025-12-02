#!/bin/bash
# ==============================================================================
# Tabrela - Health Check Script
# ==============================================================================
# Checks all backend services and returns appropriate status.
# Used by both Docker HEALTHCHECK and Railway health checks.
#
# Returns:
#   0 (success) - All services healthy
#   1 (failure) - One or more services unhealthy
#
# Output: JSON with status of each service
# ==============================================================================

set -e

# Service endpoints (internal ports)
AUTH_URL="http://127.0.0.1:8081/health"
ATTENDANCE_URL="http://127.0.0.1:8082/health"
MERIT_URL="http://127.0.0.1:8083/health"
EMAIL_URL="http://127.0.0.1:5000/health"

# Track overall health
all_healthy=true
results=""

check_service() {
    local name=$1
    local url=$2
    local status="unhealthy"
    
    if curl -sf --max-time 2 "$url" > /dev/null 2>&1; then
        status="healthy"
    else
        all_healthy=false
    fi
    
    if [ -n "$results" ]; then
        results="$results,"
    fi
    results="$results\"$name\":\"$status\""
}

# Check all services
check_service "auth" "$AUTH_URL"
check_service "attendance" "$ATTENDANCE_URL"
check_service "merit" "$MERIT_URL"
check_service "email" "$EMAIL_URL"

# Output JSON
if [ "$all_healthy" = true ]; then
    echo "{\"status\":\"healthy\",\"services\":{$results}}"
    exit 0
else
    echo "{\"status\":\"unhealthy\",\"services\":{$results}}"
    exit 1
fi
