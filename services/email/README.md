# Email Service

A Python Flask microservice for sending transactional emails using Resend API.

## Features

- Email verification emails
- Password reset emails
- Welcome emails
- Localhost-only CORS for security
- Service-to-service authentication via API keys

## Setup

1. Install dependencies:
```bash
pip install -r requirements.txt
```

2. Configure environment variables:
```bash
cp .env.example .env
# Edit .env with your configuration
```

3. Run the service:
```bash
python app.py
```

Or with gunicorn:
```bash
gunicorn --bind 0.0.0.0:5000 --workers 2 app:app
```

## Docker

Build and run with Docker:
```bash
docker build -t email-service .
docker run -p 5000:5000 --env-file .env email-service
```

## API Endpoints

All endpoints require the `X-API-Key` header for authentication.

### POST /api/send-verification-email
Send an email verification link.

**Request:**
```json
{
  "to_email": "user@example.com",
  "username": "johndoe",
  "verification_token": "abc123..."
}
```

### POST /api/send-password-reset-email
Send a password reset link.

**Request:**
```json
{
  "to_email": "user@example.com",
  "username": "johndoe",
  "reset_token": "xyz789..."
}
```

### POST /api/send-welcome-email
Send a welcome email after verification.

**Request:**
```json
{
  "to_email": "user@example.com",
  "username": "johndoe"
}
```

### GET /health
Health check endpoint (no authentication required).

## Security

- CORS is configured to only accept requests from localhost/127.0.0.1
- All API endpoints require a shared API key
- Suitable for same-server/VPS deployments only
