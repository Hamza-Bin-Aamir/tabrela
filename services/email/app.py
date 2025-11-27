from flask import Flask, request, jsonify
from flask_cors import CORS
import resend
import os
from dotenv import load_dotenv
import logging
from pydantic import ValidationError

# Import Pydantic models
from models import (
    VerificationEmailRequest,
    PasswordResetEmailRequest,
    WelcomeEmailRequest,
    EmailResponse,
    ErrorResponse,
    HealthResponse
)

# Load environment variables
load_dotenv()

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

app = Flask(__name__)

# Configure CORS to only allow localhost/127.0.0.1 (same device/VPS)
CORS(app, resources={
    r"/api/*": {
        "origins": ["http://localhost:*", "http://127.0.0.1:*", "http://0.0.0.0:*"],
        "methods": ["POST"],
        "allow_headers": ["Content-Type", "Authorization"]
    }
})

# Configure Resend
resend.api_key = os.getenv("RESEND_API_KEY")
FROM_EMAIL = os.getenv("FROM_EMAIL", "onboarding@resend.dev")
FRONTEND_URL = os.getenv("FRONTEND_URL", "http://localhost:5173")

# API Key for authentication between services
SERVICE_API_KEY = os.getenv("SERVICE_API_KEY")


def verify_api_key():
    """Verify the API key from the request header"""
    api_key = request.headers.get("X-API-Key")
    if not api_key or api_key != SERVICE_API_KEY:
        return False
    return True


@app.route("/health", methods=["GET"])
def health():
    """Health check endpoint"""
    response = HealthResponse(
        status="healthy",
        service="email-service",
        version="1.0.0"
    )
    return jsonify(response.model_dump()), 200


@app.route("/api/send-verification-email", methods=["POST"])
def send_verification_email():
    """Send OTP verification email"""
    if not verify_api_key():
        error = ErrorResponse(error="Unauthorized")
        return jsonify(error.model_dump()), 401

    try:
        # Validate request data using Pydantic
        data = request.json
        validated_data = VerificationEmailRequest(**data)
        
        to_email = validated_data.to_email
        username = validated_data.username
        otp = validated_data.otp

        html = f"""
        <div style="font-family: Arial, sans-serif; max-width: 600px; margin: 0 auto;">
            <div style="background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); color: white; padding: 30px; text-align: center; border-radius: 10px 10px 0 0;">
                <h1 style="margin: 0;">Welcome to Tabrela!</h1>
            </div>
            <div style="background: #f9fafb; padding: 30px; border-radius: 0 0 10px 10px;">
                <h2 style="color: #333;">Hi {username},</h2>
                <p style="color: #333; line-height: 1.6;">Thank you for registering! Please use the following one-time password (OTP) to verify your email address:</p>
                
                <div style="background: white; border: 2px dashed #667eea; padding: 20px; text-align: center; border-radius: 10px; margin: 20px 0;">
                    <div style="font-size: 36px; font-weight: bold; letter-spacing: 8px; color: #667eea; font-family: 'Courier New', monospace;">{otp}</div>
                </div>
                
                <p style="color: #333;"><strong>This code will expire in 10 minutes.</strong></p>
                <p style="color: #333;">If you didn't create an account, you can safely ignore this email.</p>
                
                <div style="text-align: center; margin-top: 30px; color: #6b7280; font-size: 12px;">
                    <p>&copy; 2025 Tabrela. All rights reserved.</p>
                </div>
            </div>
        </div>
        """

        response = resend.Emails.send({
            "from": FROM_EMAIL,
            "to": to_email,
            "subject": "Verify Your Email Address - OTP Code",
            "html": html
        })

        logger.info(f"Verification email sent to {to_email}")
        email_response = EmailResponse(
            success=True,
            email_id=response.get("id"),
            message="Verification email sent successfully"
        )
        return jsonify(email_response.model_dump()), 200

    except ValidationError as e:
        logger.error(f"Validation error: {e.errors()}")
        error = ErrorResponse(
            error="Validation error",
            details={"errors": e.errors()}
        )
        return jsonify(error.model_dump()), 400
    except Exception as e:
        logger.error(f"Error sending verification email: {str(e)}")
        error = ErrorResponse(error=str(e))
        return jsonify(error.model_dump()), 500


@app.route("/api/send-password-reset-email", methods=["POST"])
def send_password_reset_email():
    """Send password reset OTP email"""
    if not verify_api_key():
        error = ErrorResponse(error="Unauthorized")
        return jsonify(error.model_dump()), 401

    try:
        # Validate request data using Pydantic
        data = request.json
        validated_data = PasswordResetEmailRequest(**data)
        
        to_email = validated_data.to_email
        username = validated_data.username
        otp = validated_data.otp

        html = f"""
        <div style="font-family: Arial, sans-serif; max-width: 600px; margin: 0 auto;">
            <div style="background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); color: white; padding: 30px; text-align: center; border-radius: 10px 10px 0 0;">
                <h1 style="margin: 0;">🔒 Password Reset Request</h1>
            </div>
            <div style="background: #f9fafb; padding: 30px; border-radius: 0 0 10px 10px;">
                <h2 style="color: #333;">Hi {username},</h2>
                <p style="color: #333; line-height: 1.6;">We received a request to reset your password. Use the OTP below to reset your password:</p>
                <div style="text-align: center; margin: 30px 0;">
                    <div style="display: inline-block; background: white; padding: 20px 40px; border: 2px dashed #667eea; border-radius: 8px; font-size: 32px; font-weight: bold; letter-spacing: 8px; color: #333;">{otp}</div>
                </div>
                <p style="color: #333; line-height: 1.6;">This OTP will expire in <strong>10 minutes</strong> and can only be used once. You have <strong>5 attempts</strong> to enter it correctly.</p>
                <div style="background: #fef2f2; border-left: 4px solid #ef4444; padding: 15px; margin: 20px 0;">
                    <strong style="color: #333;">Security Notice:</strong> <span style="color: #333;">If you didn't request a password reset, please ignore this email or contact support if you're concerned about your account security.</span>
                </div>
                
                <div style="text-align: center; margin-top: 30px; color: #6b7280; font-size: 12px;">
                    <p>&copy; 2025 Tabrela. All rights reserved.</p>
                </div>
            </div>
        </div>
        """

        response = resend.Emails.send({
            "from": FROM_EMAIL,
            "to": to_email,
            "subject": "Reset Your Password - OTP Code",
            "html": html
        })

        logger.info(f"Password reset OTP email sent to {to_email}")
        email_response = EmailResponse(
            success=True,
            email_id=response.get("id"),
            message="Password reset email sent successfully"
        )
        return jsonify(email_response.model_dump()), 200

    except ValidationError as e:
        logger.error(f"Validation error: {e.errors()}")
        error = ErrorResponse(
            error="Validation error",
            details={"errors": e.errors()}
        )
        return jsonify(error.model_dump()), 400
    except Exception as e:
        logger.error(f"Error sending password reset email: {str(e)}")
        error = ErrorResponse(error=str(e))
        return jsonify(error.model_dump()), 500


@app.route("/api/send-welcome-email", methods=["POST"])
def send_welcome_email():
    """Send welcome email after email verification"""
    if not verify_api_key():
        error = ErrorResponse(error="Unauthorized")
        return jsonify(error.model_dump()), 401

    try:
        # Validate request data using Pydantic
        data = request.json
        validated_data = WelcomeEmailRequest(**data)
        
        to_email = validated_data.to_email
        username = validated_data.username

        html = f"""
        <div style="font-family: Arial, sans-serif; max-width: 600px; margin: 0 auto;">
            <div style="background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); color: white; padding: 30px; text-align: center; border-radius: 10px 10px 0 0;">
                <h1 style="margin: 0;">🎉 Welcome to Tabrela!</h1>
            </div>
            <div style="background: #f9fafb; padding: 30px; border-radius: 0 0 10px 10px;">
                <h2 style="color: #333;">Hi {username},</h2>
                <p style="color: #333; line-height: 1.6;">Your email has been successfully verified! You now have full access to your Tabrela account.</p>
                <p style="color: #333;">We're excited to have you on board!</p>
                <div style="text-align: center; margin: 20px 0;">
                    <a href="{FRONTEND_URL}" style="display: inline-block; background: #667eea; color: white; padding: 15px 30px; text-decoration: none; border-radius: 5px;">Go to Dashboard</a>
                </div>
                <p style="color: #333;">If you have any questions, feel free to reach out to our support team.</p>
                
                <div style="text-align: center; margin-top: 30px; color: #6b7280; font-size: 12px;">
                    <p>&copy; 2025 Tabrela. All rights reserved.</p>
                </div>
            </div>
        </div>
        """

        response = resend.Emails.send({
            "from": FROM_EMAIL,
            "to": to_email,
            "subject": "Welcome to Tabrela!",
            "html": html
        })

        logger.info(f"Welcome email sent to {to_email}")
        email_response = EmailResponse(
            success=True,
            email_id=response.get("id"),
            message="Welcome email sent successfully"
        )
        return jsonify(email_response.model_dump()), 200

    except ValidationError as e:
        logger.error(f"Validation error: {e.errors()}")
        error = ErrorResponse(
            error="Validation error",
            details={"errors": e.errors()}
        )
        return jsonify(error.model_dump()), 400
    except Exception as e:
        logger.error(f"Error sending welcome email: {str(e)}")
        error = ErrorResponse(error=str(e))
        return jsonify(error.model_dump()), 500


if __name__ == "__main__":
    port = int(os.getenv("PORT", 5000))
    debug = os.getenv("DEBUG", "False").lower() == "true"
    
    logger.info(f"Starting email service on port {port}")
    logger.info(f"CORS configured for localhost only")
    
    app.run(host="0.0.0.0", port=port, debug=debug)
