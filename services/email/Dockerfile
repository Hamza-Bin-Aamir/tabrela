# ==============================================================================
# Email Service - Standalone Development Dockerfile
# ==============================================================================
# NOTE: This Dockerfile is for LOCAL DEVELOPMENT ONLY.
# 
# For production deployment, use: docker/Dockerfile.backend
# which builds all services (auth, attendance, merit, email) into a single image.
#
# Usage (development):
#   docker build -t tabrela-email -f services/email/Dockerfile services/email/
#   docker run -p 5000:5000 --env-file .env tabrela-email
# ==============================================================================

FROM python:3.11-slim

# Set working directory
WORKDIR /app

# Install dependencies
COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

# Copy application code
COPY app.py .
COPY models.py .

# Expose port (internal use only - do NOT expose to public internet)
EXPOSE 5000

# Health check (uses Python since curl isn't needed in dev)
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD python -c "import urllib.request; urllib.request.urlopen('http://localhost:5000/health')" || exit 1

# Run with gunicorn WSGI server
# Note: Binds to 0.0.0.0 for container network access
# The port should NOT be exposed to public internet - only internal services
CMD ["gunicorn", "--bind", "0.0.0.0:5000", "--workers", "2", "--timeout", "120", "app:app"]
