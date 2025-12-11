use chrono::Utc;
use pasetors::{
    claims::ClaimsValidationRules, keys::SymmetricKey, local, token::UntrustedToken, version4::V4,
    Local,
};

use crate::models::Claims;

pub fn validate_paseto_token(token: &str, secret: &str) -> Result<Claims, String> {
    // Ensure the secret is exactly 32 bytes for V4
    if secret.len() != 32 {
        return Err(format!(
            "Secret key must be exactly 32 bytes, got {}",
            secret.len()
        ));
    }

    let symmetric_key =
        SymmetricKey::<V4>::from(secret.as_bytes()).map_err(|e| format!("Key error: {}", e))?;

    let validation_rules = ClaimsValidationRules::new();
    let untrusted_token =
        UntrustedToken::<Local, V4>::try_from(token).map_err(|e| format!("Parse error: {}", e))?;

    let trusted_token = local::decrypt(
        &symmetric_key,
        &untrusted_token,
        &validation_rules,
        None,
        None,
    )
    .map_err(|e| format!("Decryption error: {}", e))?;

    let claims_string = trusted_token
        .payload_claims()
        .ok_or_else(|| "No claims in token".to_string())?
        .to_string()
        .map_err(|e| format!("Claims serialization error: {}", e))?;

    let payload: serde_json::Value =
        serde_json::from_str(&claims_string).map_err(|e| format!("JSON parse error: {}", e))?;

    let sub = payload
        .get("sub")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing 'sub' claim".to_string())?
        .to_string();

    let username = payload
        .get("username")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing 'username' claim".to_string())?
        .to_string();

    let exp_str = payload
        .get("exp")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing 'exp' claim".to_string())?;

    let exp = chrono::DateTime::parse_from_rfc3339(exp_str)
        .map_err(|e| format!("Invalid expiration format: {}", e))?
        .timestamp();

    let iat = payload
        .get("iat")
        .and_then(|v| v.as_i64())
        .ok_or_else(|| "Missing 'iat' claim".to_string())?;

    let jti = payload
        .get("jti")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing 'jti' claim".to_string())?
        .to_string();

    let token_type = payload
        .get("token_type")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing 'token_type' claim".to_string())?
        .to_string();

    // Check if token has expired
    let now = Utc::now().timestamp();
    if exp < now {
        return Err("Token has expired".to_string());
    }

    Ok(Claims {
        sub,
        username,
        exp,
        iat,
        jti,
        token_type,
    })
}
