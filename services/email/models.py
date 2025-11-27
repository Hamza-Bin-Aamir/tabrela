"""
Pydantic models for email service request validation.
Enforces strict data types and validation for all incoming requests.
"""

from pydantic import BaseModel, EmailStr, Field, field_validator
from typing import Optional
import re


class VerificationEmailRequest(BaseModel):
    """Model for email verification request"""
    to_email: EmailStr = Field(
        ...,
        description="Recipient email address",
        examples=["user@example.com"]
    )
    username: str = Field(
        ...,
        min_length=3,
        max_length=50,
        description="Username of the recipient",
        examples=["johndoe"]
    )
    otp: str = Field(
        ...,
        min_length=6,
        max_length=6,
        pattern=r"^\d{6}$",
        description="6-digit OTP code",
        examples=["123456"]
    )

    @field_validator('otp')
    @classmethod
    def validate_otp(cls, v: str) -> str:
        """Validate that OTP is exactly 6 digits"""
        if not re.match(r"^\d{6}$", v):
            raise ValueError("OTP must be exactly 6 digits")
        return v

    @field_validator('username')
    @classmethod
    def validate_username(cls, v: str) -> str:
        """Validate username format"""
        if not v.strip():
            raise ValueError("Username cannot be empty or whitespace only")
        return v.strip()

    model_config = {
        "str_strip_whitespace": True,
        "json_schema_extra": {
            "examples": [
                {
                    "to_email": "user@example.com",
                    "username": "johndoe",
                    "otp": "123456"
                }
            ]
        }
    }


class PasswordResetEmailRequest(BaseModel):
    """Model for password reset email request"""
    to_email: EmailStr = Field(
        ...,
        description="Recipient email address",
        examples=["user@example.com"]
    )
    username: str = Field(
        ...,
        min_length=3,
        max_length=50,
        description="Username of the recipient",
        examples=["johndoe"]
    )
    otp: str = Field(
        ...,
        min_length=6,
        max_length=6,
        pattern=r"^\d{6}$",
        description="6-digit OTP code for password reset",
        examples=["123456"]
    )

    @field_validator('otp')
    @classmethod
    def validate_otp(cls, v: str) -> str:
        """Validate that OTP is exactly 6 digits"""
        if not re.match(r"^\d{6}$", v):
            raise ValueError("OTP must be exactly 6 digits")
        return v

    @field_validator('username')
    @classmethod
    def validate_username(cls, v: str) -> str:
        """Validate username format"""
        if not v.strip():
            raise ValueError("Username cannot be empty or whitespace only")
        return v.strip()

    model_config = {
        "str_strip_whitespace": True,
        "json_schema_extra": {
            "examples": [
                {
                    "to_email": "user@example.com",
                    "username": "johndoe",
                    "otp": "987654"
                }
            ]
        }
    }


class WelcomeEmailRequest(BaseModel):
    """Model for welcome email request"""
    to_email: EmailStr = Field(
        ...,
        description="Recipient email address",
        examples=["user@example.com"]
    )
    username: str = Field(
        ...,
        min_length=3,
        max_length=50,
        description="Username of the recipient",
        examples=["johndoe"]
    )

    @field_validator('username')
    @classmethod
    def validate_username(cls, v: str) -> str:
        """Validate username format"""
        if not v.strip():
            raise ValueError("Username cannot be empty or whitespace only")
        return v.strip()

    model_config = {
        "str_strip_whitespace": True,
        "json_schema_extra": {
            "examples": [
                {
                    "to_email": "user@example.com",
                    "username": "johndoe"
                }
            ]
        }
    }


class EmailResponse(BaseModel):
    """Model for successful email response"""
    success: bool = Field(
        default=True,
        description="Indicates if the email was sent successfully"
    )
    email_id: Optional[str] = Field(
        default=None,
        description="Email ID from the email service provider"
    )
    message: Optional[str] = Field(
        default=None,
        description="Additional message or information"
    )

    model_config = {
        "json_schema_extra": {
            "examples": [
                {
                    "success": True,
                    "email_id": "abc123-def456",
                    "message": "Email sent successfully"
                }
            ]
        }
    }


class ErrorResponse(BaseModel):
    """Model for error response"""
    error: str = Field(
        ...,
        description="Error message describing what went wrong"
    )
    details: Optional[dict] = Field(
        default=None,
        description="Additional error details"
    )

    model_config = {
        "json_schema_extra": {
            "examples": [
                {
                    "error": "Invalid email format",
                    "details": {"field": "to_email", "issue": "not a valid email"}
                }
            ]
        }
    }


class HealthResponse(BaseModel):
    """Model for health check response"""
    status: str = Field(
        default="healthy",
        description="Health status of the service"
    )
    service: str = Field(
        default="email-service",
        description="Name of the service"
    )
    version: str = Field(
        default="1.0.0",
        description="Service version"
    )

    model_config = {
        "json_schema_extra": {
            "examples": [
                {
                    "status": "healthy",
                    "service": "email-service",
                    "version": "1.0.0"
                }
            ]
        }
    }
