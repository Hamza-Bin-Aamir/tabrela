use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Debug, Serialize)]
struct SendVerificationEmailRequest {
    to_email: String,
    username: String,
    otp: String,
}

#[derive(Debug, Serialize)]
struct SendPasswordResetEmailRequest {
    to_email: String,
    username: String,
    otp: String,
}

#[derive(Debug, Serialize)]
struct SendWelcomeEmailRequest {
    to_email: String,
    username: String,
}

#[derive(Debug, Deserialize)]
struct EmailResponse {
    success: bool,
    email_id: Option<String>,
}

pub struct EmailClient {
    base_url: String,
    api_key: String,
    client: reqwest::Client,
}

impl EmailClient {
    pub fn new(base_url: String, api_key: String) -> Self {
        Self {
            base_url,
            api_key,
            client: reqwest::Client::new(),
        }
    }

    pub async fn send_verification_email(
        &self,
        to_email: &str,
        username: &str,
        otp: &str,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let request = SendVerificationEmailRequest {
            to_email: to_email.to_string(),
            username: username.to_string(),
            otp: otp.to_string(),
        };

        self.send_email_request("/api/send-verification-email", &request)
            .await
    }

    pub async fn send_password_reset_email(
        &self,
        to_email: &str,
        username: &str,
        otp: &str,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let request = SendPasswordResetEmailRequest {
            to_email: to_email.to_string(),
            username: username.to_string(),
            otp: otp.to_string(),
        };

        self.send_email_request("/api/send-password-reset-email", &request)
            .await
    }

    pub async fn send_welcome_email(
        &self,
        to_email: &str,
        username: &str,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let request = SendWelcomeEmailRequest {
            to_email: to_email.to_string(),
            username: username.to_string(),
        };

        self.send_email_request("/api/send-welcome-email", &request)
            .await
    }

    async fn send_email_request<T: Serialize>(
        &self,
        endpoint: &str,
        request: &T,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let url = format!("{}{}", self.base_url, endpoint);

        let response = self
            .client
            .post(&url)
            .header("X-API-Key", &self.api_key)
            .json(request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(format!("Failed to send email: {}", error_text).into());
        }

        Ok(())
    }
}
