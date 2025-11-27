"""
Test script for Pydantic models validation.
Run this to verify all models work correctly with strict type checking.
"""

from models import (
    VerificationEmailRequest,
    PasswordResetEmailRequest,
    WelcomeEmailRequest,
    EmailResponse,
    ErrorResponse,
    HealthResponse
)
from pydantic import ValidationError
import json


def test_verification_email_request():
    """Test VerificationEmailRequest validation"""
    print("\n=== Testing VerificationEmailRequest ===")
    
    # Valid request
    try:
        valid = VerificationEmailRequest(
            to_email="user@example.com",
            username="johndoe",
            otp="123456"
        )
        print(f"✓ Valid request: {valid.model_dump()}")
    except ValidationError as e:
        print(f"✗ Unexpected validation error: {e}")
    
    # Invalid email
    try:
        invalid = VerificationEmailRequest(
            to_email="not-an-email",
            username="johndoe",
            otp="123456"
        )
        print(f"✗ Should have failed: invalid email")
    except ValidationError as e:
        print(f"✓ Caught invalid email: {e.error_count()} error(s)")
    
    # Invalid OTP (not 6 digits)
    try:
        invalid = VerificationEmailRequest(
            to_email="user@example.com",
            username="johndoe",
            otp="12345"
        )
        print(f"✗ Should have failed: invalid OTP length")
    except ValidationError as e:
        print(f"✓ Caught invalid OTP: {e.error_count()} error(s)")
    
    # Invalid OTP (not all digits)
    try:
        invalid = VerificationEmailRequest(
            to_email="user@example.com",
            username="johndoe",
            otp="12345a"
        )
        print(f"✗ Should have failed: invalid OTP format")
    except ValidationError as e:
        print(f"✓ Caught invalid OTP format: {e.error_count()} error(s)")
    
    # Empty username
    try:
        invalid = VerificationEmailRequest(
            to_email="user@example.com",
            username="",
            otp="123456"
        )
        print(f"✗ Should have failed: empty username")
    except ValidationError as e:
        print(f"✓ Caught empty username: {e.error_count()} error(s)")
    
    # Username too short
    try:
        invalid = VerificationEmailRequest(
            to_email="user@example.com",
            username="ab",
            otp="123456"
        )
        print(f"✗ Should have failed: username too short")
    except ValidationError as e:
        print(f"✓ Caught username too short: {e.error_count()} error(s)")


def test_password_reset_email_request():
    """Test PasswordResetEmailRequest validation"""
    print("\n=== Testing PasswordResetEmailRequest ===")
    
    # Valid request
    try:
        valid = PasswordResetEmailRequest(
            to_email="user@example.com",
            username="johndoe",
            otp="654321"
        )
        print(f"✓ Valid request: {valid.model_dump()}")
    except ValidationError as e:
        print(f"✗ Unexpected validation error: {e}")


def test_welcome_email_request():
    """Test WelcomeEmailRequest validation"""
    print("\n=== Testing WelcomeEmailRequest ===")
    
    # Valid request
    try:
        valid = WelcomeEmailRequest(
            to_email="user@example.com",
            username="johndoe"
        )
        print(f"✓ Valid request: {valid.model_dump()}")
    except ValidationError as e:
        print(f"✗ Unexpected validation error: {e}")
    
    # Missing field
    try:
        invalid = WelcomeEmailRequest(
            to_email="user@example.com"
        )
        print(f"✗ Should have failed: missing username")
    except ValidationError as e:
        print(f"✓ Caught missing field: {e.error_count()} error(s)")


def test_email_response():
    """Test EmailResponse model"""
    print("\n=== Testing EmailResponse ===")
    
    response = EmailResponse(
        success=True,
        email_id="abc123-def456",
        message="Email sent successfully"
    )
    print(f"✓ EmailResponse: {response.model_dump()}")


def test_error_response():
    """Test ErrorResponse model"""
    print("\n=== Testing ErrorResponse ===")
    
    error = ErrorResponse(
        error="Invalid email format",
        details={"field": "to_email", "issue": "not a valid email"}
    )
    print(f"✓ ErrorResponse: {error.model_dump()}")


def test_health_response():
    """Test HealthResponse model"""
    print("\n=== Testing HealthResponse ===")
    
    health = HealthResponse(
        status="healthy",
        service="email-service",
        version="1.0.0"
    )
    print(f"✓ HealthResponse: {health.model_dump()}")


def test_json_serialization():
    """Test JSON serialization and deserialization"""
    print("\n=== Testing JSON Serialization ===")
    
    # Create a request
    request = VerificationEmailRequest(
        to_email="user@example.com",
        username="johndoe",
        otp="123456"
    )
    
    # Serialize to JSON
    json_str = request.model_dump_json()
    print(f"✓ Serialized to JSON: {json_str}")
    
    # Deserialize from JSON
    json_dict = json.loads(json_str)
    reconstructed = VerificationEmailRequest(**json_dict)
    print(f"✓ Deserialized from JSON: {reconstructed.model_dump()}")
    
    # Verify they're equal
    assert request.model_dump() == reconstructed.model_dump()
    print("✓ Original and reconstructed are equal")


def test_whitespace_stripping():
    """Test that whitespace is automatically stripped"""
    print("\n=== Testing Whitespace Stripping ===")
    
    request = VerificationEmailRequest(
        to_email="  user@example.com  ",
        username="  johndoe  ",
        otp="123456"
    )
    
    assert request.to_email == "user@example.com"
    assert request.username == "johndoe"
    print("✓ Whitespace stripped successfully")


if __name__ == "__main__":
    print("=" * 60)
    print("Running Pydantic Model Validation Tests")
    print("=" * 60)
    
    test_verification_email_request()
    test_password_reset_email_request()
    test_welcome_email_request()
    test_email_response()
    test_error_response()
    test_health_response()
    test_json_serialization()
    test_whitespace_stripping()
    
    print("\n" + "=" * 60)
    print("All tests completed!")
    print("=" * 60)
