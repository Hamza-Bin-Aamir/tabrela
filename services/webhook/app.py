"""
Webhook Service - Handles deployment webhooks and triggers GitHub Actions
"""
from flask import Flask, request, jsonify
import requests
import os
import logging
import hmac
import hashlib

logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

app = Flask(__name__)

# Configuration
GITHUB_TOKEN = os.getenv("GITHUB_TOKEN")
GITHUB_REPO = os.getenv("GITHUB_REPO", "Hamza-Bin-Aamir/tabrela")
WEBHOOK_SECRET = os.getenv("WEBHOOK_SECRET")  # Optional: for verifying Railway webhooks


def verify_webhook_signature(payload: bytes, signature: str) -> bool:
    """Verify webhook signature if WEBHOOK_SECRET is configured"""
    if not WEBHOOK_SECRET:
        return True  # Skip verification if no secret configured
    
    if not signature:
        return False
    
    expected = hmac.new(
        WEBHOOK_SECRET.encode(),
        payload,
        hashlib.sha256
    ).hexdigest()
    
    return hmac.compare_digest(f"sha256={expected}", signature)


@app.route("/health", methods=["GET"])
def health():
    """Health check endpoint"""
    return jsonify({
        "status": "healthy",
        "service": "webhook-service",
        "version": "1.0.0"
    }), 200


@app.route("/railway-deploy", methods=["POST"])
def railway_deploy_webhook():
    """
    Handle Railway deployment webhook.
    
    Railway sends a POST request when deployment completes.
    This endpoint triggers GitHub Actions to deploy the frontend.
    """
    # Verify signature if configured
    signature = request.headers.get("X-Signature")
    if not verify_webhook_signature(request.data, signature):
        logger.warning("Invalid webhook signature")
        return jsonify({"error": "Invalid signature"}), 401
    
    try:
        payload = request.json
        logger.info(f"Received Railway webhook: {payload}")
        
        # Railway webhook payload structure varies, handle common cases
        # Check for successful deployment
        status = (
            payload.get("deployment", {}).get("status") or
            payload.get("status") or
            payload.get("type")
        )
        
        # Only trigger on successful deployments
        # Railway uses "SUCCESS" or the event type might be "deployment.completed"
        is_success = (
            status == "SUCCESS" or
            status == "COMPLETED" or
            payload.get("type") == "deployment.completed"
        )
        
        if not is_success:
            logger.info(f"Deployment status '{status}' is not successful, skipping")
            return jsonify({
                "message": "Ignored - deployment not successful",
                "status": status
            }), 200
        
        # Extract commit SHA
        commit_sha = (
            payload.get("deployment", {}).get("meta", {}).get("commitHash") or
            payload.get("meta", {}).get("commitHash") or
            payload.get("commitHash") or
            "unknown"
        )
        
        # Trigger GitHub Actions
        if not GITHUB_TOKEN:
            logger.error("GITHUB_TOKEN not configured")
            return jsonify({"error": "GITHUB_TOKEN not configured"}), 500
        
        github_response = requests.post(
            f"https://api.github.com/repos/{GITHUB_REPO}/dispatches",
            headers={
                "Accept": "application/vnd.github.v3+json",
                "Authorization": f"token {GITHUB_TOKEN}",
                "Content-Type": "application/json",
                "User-Agent": "Tabrela-Webhook-Service"
            },
            json={
                "event_type": "railway-deploy-success",
                "client_payload": {
                    "commit_sha": commit_sha,
                    "deployment_id": payload.get("deployment", {}).get("id", "unknown"),
                    "environment": payload.get("environment", {}).get("name", "production")
                }
            },
            timeout=10
        )
        
        if github_response.status_code not in [200, 204]:
            logger.error(f"GitHub API error: {github_response.status_code} - {github_response.text}")
            return jsonify({
                "error": "Failed to trigger GitHub Actions",
                "status_code": github_response.status_code
            }), 500
        
        logger.info(f"Successfully triggered frontend deploy for commit: {commit_sha}")
        return jsonify({
            "message": "Frontend deployment triggered",
            "commit_sha": commit_sha
        }), 200
        
    except Exception as e:
        logger.error(f"Webhook handler error: {e}")
        return jsonify({"error": str(e)}), 500


if __name__ == "__main__":
    port = int(os.getenv("PORT", 5001))
    logger.info(f"Starting webhook service on port {port}")
    app.run(host="0.0.0.0", port=port)
