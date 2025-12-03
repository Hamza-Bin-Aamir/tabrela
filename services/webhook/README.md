# Webhook Service

Handles deployment webhooks from Railway and triggers GitHub Actions to deploy the frontend.

## Purpose

This service solves the coordination problem between backend and frontend deployments:
1. Railway deploys the backend
2. Railway sends a webhook to this service
3. This service triggers GitHub Actions to deploy the frontend
4. Frontend only deploys after backend is confirmed running

## Environment Variables

| Variable | Required | Description |
|----------|----------|-------------|
| `GITHUB_TOKEN` | Yes | GitHub Personal Access Token with `repo` scope |
| `GITHUB_REPO` | No | Repository in format `owner/repo` (default: `Hamza-Bin-Aamir/tabrela`) |
| `WEBHOOK_SECRET` | No | Secret for verifying Railway webhook signatures |
| `PORT` | No | Port to listen on (default: `5001`) |

## Endpoints

### `GET /health`
Health check endpoint.

### `POST /railway-deploy`
Webhook endpoint for Railway deployment notifications.

## Railway Setup

1. Go to Railway project → Settings → Webhooks
2. Add webhook URL: `https://your-backend-url/webhook/railway-deploy`
3. Select "Deploy Success" event (or equivalent)

## Local Testing

```bash
cd services/webhook
pip install -r requirements.txt
GITHUB_TOKEN=your_token python app.py
```

Test the webhook:
```bash
curl -X POST http://localhost:5001/railway-deploy \
  -H "Content-Type: application/json" \
  -d '{"deployment": {"status": "SUCCESS", "meta": {"commitHash": "abc123"}}}'
```
